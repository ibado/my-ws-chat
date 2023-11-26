package com.example.my_ws_chat_client.notifications

import android.util.Log
import com.example.my_ws_chat_client.BuildConfig
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.channels.onFailure
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.sse.EventSource
import okhttp3.sse.EventSourceListener
import okhttp3.sse.EventSources
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import kotlin.coroutines.cancellation.CancellationException

object NotificationsClient {
    private const val TAG = "NotificationsClient"

    suspend fun streamNotifications(jwt: String): Flow<Message> = coroutineScope {
        val client = OkHttpClient.Builder().connectTimeout(5, TimeUnit.SECONDS)
            .readTimeout(10, TimeUnit.MINUTES)
            .writeTimeout(10, TimeUnit.MINUTES)
            .build()
        val request = Request.Builder()
            .url("${BuildConfig.BACKEND_BASE_URL}/notifications")
            .addHeader("Accept", "text/event-stream")
            .addHeader("Authorization", jwt)
            .build()

        callbackFlow {
            val eventListener = object : EventSourceListener() {
                override fun onFailure(eventSource: EventSource, t: Throwable?, response: Response?) {
                    super.onFailure(eventSource, t, response)
                    Log.e(TAG, "Error listening notifications", t)
                    cancel(CancellationException("Error, canceling notifications client...", t))
                }

                override fun onEvent(
                    eventSource: EventSource,
                    id: String?,
                    type: String?,
                    data: String
                ) {
                    super.onEvent(eventSource, id, type, data)
                    Log.i(TAG, "onEvent(): $data")
                    val message = JSONObject(data).let {
                        Message(it.getString("addressee_nickname"), it.getString("message"))
                    }
                    trySendBlocking(message)
                        .onFailure { Log.e(TAG, "Error trying to send event!", it) }
                }

                override fun onClosed(eventSource: EventSource) {
                    super.onClosed(eventSource)
                    channel.close()
                }
            }

            val source = EventSources.createFactory(client)
                .newEventSource(request, eventListener)
            client.newCall(request).execute()
            awaitClose { source.cancel() }
        }
    }

    data class Message(val addresseeNick: String, val message: String)
}