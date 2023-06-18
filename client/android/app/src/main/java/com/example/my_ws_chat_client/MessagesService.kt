package com.example.my_ws_chat_client

import android.util.Log
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.http.HttpMethod
import io.ktor.websocket.Frame
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.isActive
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit

class MessagesService : AutoCloseable {

    private lateinit var client: HttpClient
    private val msgChannel: Channel<String> = Channel()

    suspend fun initChat(onMessage: (Message) -> Unit) {
        client = HttpClient(OkHttp) {
            install(WebSockets)
            engine {
                preconfigured = OkHttpClient.Builder()
                    .pingInterval(20, TimeUnit.SECONDS)
                    .build()
            }
        }

        client.webSocket(HttpMethod.Get, "10.0.2.2", 3000, "/messages") {
            val receive = async {
                while (isActive) {
                    incoming.receive()
                        .takeIf { it is Frame.Text }
                        ?.data
                        ?.decodeToString()
                        ?.let { msg ->
                            Log.d(javaClass.name, msg)
                            onMessage(Message(msg, MsgType.OTHER))
                        } ?: Log.w(javaClass.name, "Frame is not Text")
                }
            }

            val send = async {
                send(Frame.Text("{\"sender\": \"droid\", \"addressee\": \"python\"}"))
                for (message in msgChannel) {
                    send(Frame.Text("{\"msg\": \"$message\"}"))
                    onMessage(Message(message, MsgType.ME))
                }
            }

            awaitAll(receive, send)
        }
    }

    suspend fun sendMessage(msg: String) = msgChannel.send(msg)

    enum class MsgType {
        ME, OTHER;
    }

    data class Message(val content: String, val type: MsgType)

    override fun close() {
        msgChannel.close()
        client.close()
    }
}