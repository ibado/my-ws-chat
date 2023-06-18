package com.example.my_ws_chat_client

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import com.example.my_ws_chat_client.MessagesService.*
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme
import kotlinx.coroutines.launch

class ChatActivity : ComponentActivity() {

    private val messages = mutableStateListOf<Message>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val sender = intent.getStringExtra(SENDER_KEY) ?: "droid"
        val addressee = intent.getStringExtra(ADDRESSEE_KEY) ?: "python"

        val messagesService = MessagesService()

        lifecycleScope.launchWhenCreated {
            messagesService.use { service ->
                service.initChat(sender, addressee) { message ->
                    Log.i("DROID", "new data: $message")
                    messages.add(message)
                }
            }
        }

        setContent {
            MywschatclientTheme {
                Surface(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(10.dp),
                    color = MaterialTheme.colorScheme.background,
                ) {
                    ChatView(messages)
                    MessageBar { lifecycleScope.launch { messagesService.sendMessage(it) } }
                }
            }
        }
    }

    @Composable
    @OptIn(ExperimentalMaterial3Api::class)
    private fun MessageBar(sendMessage: (String) -> Unit) {
        Row(verticalAlignment = Alignment.Bottom) {
            var textValue by remember { mutableStateOf(TextFieldValue("")) }
            TextField(value = textValue, onValueChange = {
                textValue = it
            })
            Button(onClick = {
                if (textValue.text.isNotBlank()) {
                    sendMessage(textValue.text)
                    textValue = TextFieldValue("")
                }
            }, modifier = Modifier.padding(5.dp)) {
                Text(text = "Send")
            }
        }
    }

    @Composable
    fun ChatView(messages: SnapshotStateList<Message>, modifier: Modifier = Modifier) =
        LazyColumn(modifier = modifier, contentPadding = PaddingValues(bottom = 50.dp)) {
            items(messages.size) {
                val msg = messages[it]
                val ta = if (msg.type == MsgType.ME) TextAlign.End else TextAlign.Start
                Text(
                    msg.content,
                    textAlign = ta,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = 10.dp)
                )
            }
        }

    companion object {
        private const val SENDER_KEY = "sender-key"
        private const val ADDRESSEE_KEY = "addressee-key"

        fun intent(from: Context, sender: String, addressee: String): Intent =
            Intent(from, ChatActivity::class.java)
                .apply {
                    putExtra(SENDER_KEY, sender)
                    putExtra(ADDRESSEE_KEY, addressee)
                }
    }
}