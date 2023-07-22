package com.example.my_ws_chat_client

import android.content.Context
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import com.example.my_ws_chat_client.chat.ChatActivity
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme

class MainActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val jwt = intent.getStringExtra(JWT_KEY)!!

        setContent {
            MywschatclientTheme {
                Surface(
                    modifier = Modifier.fillMaxSize().padding(10.dp),
                    color = MaterialTheme.colorScheme.background,
                ) {
                    Column(
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        var addresseeValue by remember { mutableStateOf(TextFieldValue("")) }
                        TextField(
                            value = addresseeValue,
                            onValueChange = { addresseeValue = it },
                            modifier = Modifier.fillMaxWidth().padding(5.dp),
                            label = { Text(text = "Addressee")},
                        )
                        Button(
                            onClick = { startChatActivity(jwt, addresseeValue.text)},
                            modifier = Modifier.fillMaxWidth().padding(5.dp),
                        ) {
                            Text(text = "Start Chat")
                        }
                    }
                }
            }
        }
    }

    private fun startChatActivity(jwt: String, addressee: String) {
        val intent = ChatActivity.intent(this@MainActivity, jwt, addressee)
        startActivity(intent)
    }

    companion object {
        private const val JWT_KEY = "jwt_key"
        fun intent(context: Context, jwt: String): Intent {
            return Intent(context, MainActivity::class.java).apply {
                putExtra(JWT_KEY, jwt)
            }
        }
    }
}