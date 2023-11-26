package com.example.my_ws_chat_client.notifications

import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.os.PowerManager
import android.os.PowerManager.WakeLock
import android.os.SystemClock
import android.util.Log
import androidx.core.app.NotificationCompat
import com.example.my_ws_chat_client.R
import com.example.my_ws_chat_client.chat.ChatActivity
import com.example.my_ws_chat_client.notifications.NotificationsClient.Message
import com.example.my_ws_chat_client.sharedPreferences
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class NotificationsService : Service() {

    private var wakeLock: WakeLock? = null
    private var isStarted = false
    private val exceptionHandler = CoroutineExceptionHandler { _, throwable ->
        Log.e(TAG, "Error listening notifications!", throwable)
    }
    private lateinit var scope: CoroutineScope

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i(TAG, "Starting notification service...")
        if (isStarted) return START_STICKY
        Log.i(TAG, "Notification service is started!")
        isStarted = true
        scope = CoroutineScope(Dispatchers.IO + exceptionHandler)
        scope.launch {
            wakeLock = acquireLock()
            val notificationManager = getSystemService(NotificationManager::class.java)
            val jwt = sharedPreferences().getString("jwt", "")!!
            NotificationsClient.streamNotifications(jwt).collect { message ->
                createNotification(message, jwt).let {
                    notificationManager.notify(System.currentTimeMillis().toInt(), it)
                }
            }
            stopSelf()
            Log.i(TAG, "stopping service...")
            isStarted = false
        }

        return START_STICKY
    }

    private fun acquireLock() =
        getSystemService(PowerManager::class.java)
            .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "NotificationService:lock")
            .also { it.acquire(30 * 1000) }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Service created!")
        startForeground(1, createForegroundNotification())
        Log.d(TAG, "startForeground")
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
        val alarmManager = applicationContext.getSystemService(AlarmManager::class.java)
        alarmManager.set(
            AlarmManager.ELAPSED_REALTIME,
            SystemClock.elapsedRealtime() + 1000,
            createRestartIntent()
        )
    }

    override fun onDestroy() {
        super.onDestroy()
        if (!scope.isActive) scope.cancel()
        isStarted = false
        wakeLock?.takeIf { it.isHeld }?.release()
        Log.i(TAG, "service destroyed!")
    }

    private fun createRestartIntent(): PendingIntent =
        Intent(applicationContext, NotificationsService::class.java)
            .also { it.setPackage(packageName) }
            .let { restartIntent ->
                PendingIntent.getService(
                    this,
                    1,
                    restartIntent,
                    PendingIntent.FLAG_ONE_SHOT or PendingIntent.FLAG_IMMUTABLE
                )
            }

    private fun createNotification(message: Message, jwt: String): Notification {
        val (addressee, content) = message
        val resultIntent = Intent(this, ChatActivity::class.java).apply {
            putExtra(ChatActivity.ADDRESSEE_KEY, addressee)
            putExtra(ChatActivity.SENDER_TOKEN, jwt)
        }

        val resultPendingIntent = PendingIntent.getActivity(
            this,
            0,
            resultIntent,
            PendingIntent.FLAG_IMMUTABLE
        )
        return NotificationCompat.Builder(this@NotificationsService, CHANNEL_ID)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle("New message from $addressee")
            .setContentText(content)
            .setAutoCancel(true)
            .setContentIntent(resultPendingIntent)
            .build()
    }

    private fun createForegroundNotification(): Notification =
        NotificationCompat.Builder(this@NotificationsService, CHANNEL_ID)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle("Listening for messages...")
            .build()

    companion object {
        const val CHANNEL_ID = "messages-notification-channel"
        const val TAG = "NotificationsService"
    }
}