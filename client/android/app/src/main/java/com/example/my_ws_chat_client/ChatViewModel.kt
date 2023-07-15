package com.example.my_ws_chat_client

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.http.HttpMethod
import io.ktor.websocket.Frame
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import okhttp3.OkHttpClient
import org.json.JSONObject
import java.util.concurrent.TimeUnit

class ChatViewModel : ViewModel() {

    private var client: HttpClient = HttpClient(OkHttp) {
        install(WebSockets)
        engine {
            preconfigured = OkHttpClient.Builder()
                .pingInterval(20, TimeUnit.SECONDS)
                .build()
        }
    }

    private val msgChannel: Channel<String> = Channel()
    private val mutableStateFlow = MutableStateFlow(mutableListOf<Message>())
    private var isConnected = false

    fun startChat(sender: String, addressee: String) {
        if (isConnected) return
        viewModelScope.launch {
            client.webSocket(HttpMethod.Get, HOST, PORT, PATH) {
                val receive = async { receiveMessages(this@webSocket, sender) }
                val send = async { sendMessages(this@webSocket, sender, addressee) }
                isConnected = true
                awaitAll(receive, send)
            }
        }
    }

    fun getMessages(): StateFlow<List<Message>> = mutableStateFlow

    fun sendMessage(msg: String) = viewModelScope.launch {
        msgChannel.send(msg)
    }

    override fun onCleared() {
        msgChannel.close()
        client.close()
    }

    private suspend fun sendMessages(
        socket: DefaultClientWebSocketSession,
        sender: String,
        addressee: String,
    ) {
        socket.send(Frame.Text("{\"sender\": \"$sender\", \"addressee\": \"$addressee\"}"))
        for (msgContent in msgChannel) {
            Log.d(javaClass.name, "sending msg...")
            val json = JSONObject().apply {
                put("msg", msgContent)
                put("author", sender)
            }
            socket.send(Frame.Text(json.toString()))
        }
    }

    private suspend fun CoroutineScope.receiveMessages(
        socket: DefaultClientWebSocketSession,
        sender: String,
    ) {
        while (isActive) {
            Log.d(javaClass.name, "receiving msg...")
            socket.incoming.receive()
                .takeIf { it is Frame.Text }
                ?.data
                ?.decodeToString()
                ?.let { textFrame ->
                    Log.d(javaClass.name, textFrame)
                    val (payload, author) = JSONObject(textFrame).let { json ->
                        json.get("msg") as String to json.get("author") as String
                    }
                    val type = if(author == sender) MsgType.ME else MsgType.OTHER
                    val message = Message(payload, type)
                    mutableStateFlow.emitNewMsg(message)
                } ?: Log.w(javaClass.name, "Frame is not Text")
        }
    }

    private suspend fun MutableStateFlow<MutableList<Message>>.emitNewMsg(message: Message) =
        emit(value.plus(message).toMutableList())

    companion object {
        private const val HOST = "10.0.2.2"
        private const val PORT = 3000
        private const val PATH = "/chat"
    }
}
