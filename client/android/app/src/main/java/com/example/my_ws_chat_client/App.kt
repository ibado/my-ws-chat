package com.example.my_ws_chat_client

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import com.example.my_ws_chat_client.notifications.NotificationsWorker
import java.time.Duration

class App : Application() {

    override fun onCreate() {
        super.onCreate()

        NotificationChannel(
            CHANNEL_ID,
            "Main Channel",
            NotificationManager.IMPORTANCE_DEFAULT
        ).let { channel ->
            getSystemService(NotificationManager::class.java)
                ?.createNotificationChannel(channel)
        }

        val request = PeriodicWorkRequestBuilder<NotificationsWorker>(Duration.ofMinutes(15))
            .build()
        WorkManager.getInstance(this).enqueue(request)
    }

    companion object {
        const val CHANNEL_ID = "messages-notification-channel"
    }
}