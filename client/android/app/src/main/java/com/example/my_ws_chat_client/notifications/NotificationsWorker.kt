package com.example.my_ws_chat_client.notifications

import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationCompat.BigTextStyle
import androidx.work.CoroutineWorker
import androidx.work.ForegroundInfo
import androidx.work.WorkerParameters
import com.example.my_ws_chat_client.App.Companion.CHANNEL_ID
import com.example.my_ws_chat_client.BuildConfig
import com.example.my_ws_chat_client.R
import com.example.my_ws_chat_client.chat.ChatActivity
import com.example.my_ws_chat_client.sharedPreferences
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONArray

class NotificationsWorker(
    private val context: Context,
    workerParams: WorkerParameters
) : CoroutineWorker(context, workerParams) {

    private val sharedPreferences = context.sharedPreferences()

    private val client = OkHttpClient()

    override suspend fun doWork(): Result {
        setForeground(ForegroundInfo(666, NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle("MyWsChat :)")
            .setContentText("Checking for messages...")
            .setOngoing(true)
            .build()))

        val jwt = sharedPreferences.getString("jwt", null)
            ?: return Result.failure()
        val request: Request = Request.Builder()
            .url("${BuildConfig.BACKEND_BASE_URL}/messages")
            .header("Authorization", "Bearer $jwt")
            .build()
        return client.newCall(request)
            .execute()
            .use { response ->
                response.body?.string()
                    ?.takeUnless { it.isEmpty() || !response.isSuccessful }
                    ?.let(::JSONArray)
                    ?.let { sendNotifications(it, jwt) }
                    ?: run {
                        Log.e("Notifications", "error polling messages!")
                        Result.failure()
                    }
            }
    }

    private fun sendNotifications(responseBody: JSONArray, jwt: String): Result {
        val notifications = mutableListOf<Pair<String, Notification>>()
        for (i in 0 until responseBody.length()) {
            val obj = responseBody.optJSONObject(i)
            val nickname = obj.getString("nickname")
            val message = buildString {
                val messages = obj.getJSONArray("messages")
                for (m in 0 until messages.length()) {
                    append(messages.getString(m))
                    if (m != messages.length() - 1) {
                        appendLine()
                    }
                }
            }
            createNotification(nickname, message, jwt)
                .let { nickname to it }
                .let(notifications::add)
        }

        val notificationManager = context.getSystemService(NotificationManager::class.java)
        notifications.forEach {(id, notification) ->
            notificationManager.notify(id.hashCode(), notification)
        }
        return Result.success()
    }

    private fun createNotification(
        addressee: String,
        content: String,
        jwt: String
    ): Notification {
        val resultIntent = Intent(context, ChatActivity::class.java).apply {
            putExtra(ChatActivity.ADDRESSEE_KEY, addressee)
            putExtra(ChatActivity.SENDER_TOKEN, jwt)
        }

        val resultPendingIntent = PendingIntent.getActivity(
            context,
            0,
            resultIntent,
            PendingIntent.FLAG_IMMUTABLE
        )
        return NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle("New message from $addressee")
            .setAutoCancel(true)
            .setOnlyAlertOnce(true)
            .setStyle(BigTextStyle().bigText(content))
            .setContentIntent(resultPendingIntent)
            .setPriority(NotificationManager.IMPORTANCE_HIGH)
            .build()
    }
}