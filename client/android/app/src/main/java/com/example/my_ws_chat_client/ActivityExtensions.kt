package com.example.my_ws_chat_client

import android.content.SharedPreferences
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

fun ComponentActivity.showToast(msg: CharSequence) =
    Toast.makeText(this, msg, Toast.LENGTH_SHORT).show()

fun ComponentActivity.sharedPreferences(): SharedPreferences {
    val masterKey: MasterKey = MasterKey.Builder(applicationContext)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    return EncryptedSharedPreferences.create(
        applicationContext,
        "secret_shared_prefs",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )
}

private const val RECENT_CHATS = "recent-chats"

fun ComponentActivity.saveRecentChat(addressee: String): Unit =
    sharedPreferences()
        .let { preferences ->
            val recentChats = preferences.getStringSet(RECENT_CHATS, null).orEmpty()
            preferences.edit()
                .putStringSet(RECENT_CHATS, recentChats.plus(addressee))
                .apply()
        }

fun ComponentActivity.removeAddresseeFromRecentChats(addressee: String): Unit =
    sharedPreferences().let { preferences ->
        preferences.getStringSet(RECENT_CHATS, null)
            ?.minus(addressee)
            ?.let { newSet ->
                preferences.edit().putStringSet(
                    RECENT_CHATS,
                    newSet
                ).apply()
            }
    }

fun ComponentActivity.getRecentChats() = sharedPreferences()
    .getStringSet(RECENT_CHATS, null)
    .orEmpty()