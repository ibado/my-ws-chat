package com.example.my_ws_chat_client

import android.widget.Toast
import androidx.activity.ComponentActivity

fun ComponentActivity.showToast(msg: CharSequence) =
    Toast.makeText(this, msg, Toast.LENGTH_SHORT).show()