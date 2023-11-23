package com.example.my_ws_chat_client.notifications

import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.os.PowerManager
import android.os.PowerManager.WakeLock
import android.util.Log
import androidx.core.app.NotificationCompat
import com.example.my_ws_chat_client.R
import com.example.my_ws_chat_client.chat.ChatActivity
import com.example.my_ws_chat_client.sharedPreferences
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import org.json.JSONObject

class NotificationsService : Service() {

    private var wakeLock: WakeLock? = null
    private lateinit var job: Job
    private var isStarted = false
    private val exceptionHandler = CoroutineExceptionHandler { _, throwable ->
        Log.e(TAG, "Error listening notifications!", throwable)
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i(TAG, "Starting notification service...")
        if (isStarted) return START_STICKY
        job = Job()
        Log.i(TAG, "Notification service is started!")
        isStarted = true
        CoroutineScope(Dispatchers.IO + job + exceptionHandler).launch {
            wakeLock = getSystemService(PowerManager::class.java)
                .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "NotificationService:lock")
                .also { it.acquire(30 * 1000) }

            val notificationManager = getSystemService(NotificationManager::class.java)
            val jwt = sharedPreferences().getString("jwt", "")!!
            NotificationsClient.streamNotifications(jwt).collect { eventData ->
                val (addressee, msg) = JSONObject(eventData).let {
                    it.getString("addressee_nickname") to it.getString("message")
                }
                notification(addressee, msg, jwt)
                    .let {
                        notificationManager.notify(System.currentTimeMillis().toInt(), it)
                    }
            }

            stopSelf()
            Log.i(TAG, "stopping service...")
            isStarted = false
        }

        return START_STICKY
    }

    private fun notification(addressee: String, msg: String, jwt: String): Notification {
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
            .setContentText(msg)
            .setContentIntent(resultPendingIntent)
            .build()
    }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Service created!")
        startForeground(1, createForegroundNotification())
        Log.d(TAG, "startForeground")
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
    }


    override fun onDestroy() {
        super.onDestroy()
        if (!job.isCancelled) job.cancel()
        isStarted = false
        wakeLock?.takeIf { it.isHeld }?.release()
        Log.i(TAG, "service destroyed!")
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