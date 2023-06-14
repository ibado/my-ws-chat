package com.example.my_ws_chat_client

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MywschatclientTheme {
                Surface(
                    modifier = Modifier.fillMaxSize().padding(10.dp),
                    color = MaterialTheme.colorScheme.background
                ) {
                    ChatView(
                        listOf(
                            Message("Hello there!", MsgType.ME),
                            Message("How're you?", MsgType.ME),
                            Message("I'm doing well", MsgType.OTHER),
                            Message("What about you?", MsgType.OTHER)
                        )
                    )
                }
            }
        }
    }
}

enum class MsgType {
    ME, OTHER;
}

data class Message(val content: String, val type: MsgType)

@Composable
fun ChatView(messages: List<Message>, modifier: Modifier = Modifier) =
    LazyColumn(modifier = modifier) {
        items(messages.size) {
            val msg = messages[it]
            val ta = if (msg.type == MsgType.ME) TextAlign.End else TextAlign.Start
            Text(
                msg.content,
                textAlign = ta,
                modifier = Modifier.fillMaxWidth().padding(bottom = 10.dp)
            )
        }
    }