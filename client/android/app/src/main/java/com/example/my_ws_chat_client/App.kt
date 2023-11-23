package com.example.my_ws_chat_client

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import com.example.my_ws_chat_client.notifications.NotificationsService

class App : Application() {

    override fun onCreate() {
        super.onCreate()

        NotificationChannel(
            NotificationsService.CHANNEL_ID,
            "Main Channel",
            NotificationManager.IMPORTANCE_DEFAULT
        ).let { channel ->
            getSystemService(NotificationManager::class.java)
                ?.createNotificationChannel(channel)
        }
    }
}