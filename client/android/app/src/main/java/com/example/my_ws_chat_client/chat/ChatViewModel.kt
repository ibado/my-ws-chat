package com.example.my_ws_chat_client.chat

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.left
import arrow.core.right
import com.example.my_ws_chat_client.Message
import com.example.my_ws_chat_client.MsgType
import com.example.my_ws_chat_client.chat.Response.ChatInitFailure
import com.example.my_ws_chat_client.chat.Response.ChatInitSuccess
import com.example.my_ws_chat_client.chat.Response.Msg
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.http.HttpMethod
import io.ktor.websocket.Frame
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.*
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit

private typealias InitChatError = String

class ChatViewModel : ViewModel() {

    private val msgChannel: Channel<String> = Channel()
    private val messagesFlow = MutableStateFlow(mutableListOf<Message>())
    private val errorFlow = MutableStateFlow<InitChatError?>(null)
    private var isConnected = false

    private var client: HttpClient = HttpClient(OkHttp) {
        install(WebSockets)
        engine {
            preconfigured = OkHttpClient.Builder()
                .pingInterval(20, TimeUnit.SECONDS)
                .build()
        }
    }

    private val coroutineExceptionHandler = CoroutineExceptionHandler { _, throwable ->
        Log.e(javaClass.name, "ups: ${throwable.message}", throwable)
        viewModelScope.launch {
            errorFlow.emit("Error trying to start the chat!")
        }
    }

    fun startChat(jwt: String, addressee: String) {
        if (isConnected) return
        viewModelScope.launch(coroutineExceptionHandler) {
            client.webSocket(
                method = HttpMethod.Get,
                host = HOST,
                port = PORT,
                path = PATH,
                request = {
                    headers.append("Authorization", "Bearer $jwt")
                }
            ) {
                initChat(this@webSocket, addressee).fold(
                    ifLeft = { errorFlow.emit(it) },
                    ifRight = {
                        val receive = async { receiveMessages(this@webSocket) }
                        val send = async { sendMessages(this@webSocket) }
                        isConnected = true
                        awaitAll(receive, send)
                    }
                )
            }
        }
    }

    fun getMessages(): StateFlow<List<Message>> = messagesFlow

    fun getError(): StateFlow<InitChatError?> = errorFlow

    fun sendMessage(msg: String) = viewModelScope.launch {
        msgChannel.send(msg)
    }

    override fun onCleared() {
        msgChannel.close()
        client.close()
    }

    private suspend fun initChat(
        socket: DefaultClientWebSocketSession,
        addressee: String
    ): Either<InitChatError, Unit> {
        val starChatMsg = Json.encodeToString<Request>(Request.InitChat(addressee))
        socket.send(Frame.Text(starChatMsg))
        return withTimeoutOrNull(INIT_CHAT_TIMEOUT_MS) {
            socket.getDecodedResponseOrNull()?.let { response ->
                when (response) {
                    is ChatInitFailure -> response.error.left()
                    ChatInitSuccess -> Unit.right()
                    is Msg -> "Unexpected Message!".left()
                }
            } ?: "Frame is not text!".left()
        } ?: "Server timeout".left()
    }

    private suspend fun sendMessages(
        socket: DefaultClientWebSocketSession,
    ) {
        for (msgContent in msgChannel) {
            Log.d(javaClass.name, "sending msg...")
            val msg = Json.encodeToString<Request>(Request.Msg(msgContent))
            socket.send(Frame.Text(msg))
        }
    }

    private suspend fun CoroutineScope.receiveMessages(
        socket: DefaultClientWebSocketSession,
    ) {
        while (isActive) {
            socket.getDecodedResponseOrNull()?.let { response ->
                when (response) {
                    is ChatInitFailure, ChatInitSuccess ->
                        Log.e(javaClass.name, "Wrong msg, the chat is already initiated!")

                    is Msg -> {
                        val type = if (response.isSender) MsgType.ME else MsgType.OTHER
                        val message = Message(response.msg, type)
                        messagesFlow.emitNewMsg(message)
                    }
                }
            } ?: Log.w(javaClass.name, "Frame is not Text")
        }
    }

    private suspend fun DefaultClientWebSocketSession.getDecodedResponseOrNull(
    ): Response? = incoming.receive()
        .let { frame ->
            frame.takeIf { it is Frame.Text }
                ?.data
                ?.decodeToString()
                ?.let { Json.decodeFromString(it) }
        }


    private suspend fun MutableStateFlow<MutableList<Message>>.emitNewMsg(message: Message) =
        emit(value.plus(message).toMutableList())

    companion object {
        private const val HOST = "10.0.2.2"
        private const val PORT = 3000
        private const val PATH = "/chat"
        private const val INIT_CHAT_TIMEOUT_MS: Long = 1000
    }
}
