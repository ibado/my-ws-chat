package com.example.my_ws_chat_client.chat

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import com.example.my_ws_chat_client.Message
import com.example.my_ws_chat_client.MsgType
import com.example.my_ws_chat_client.chat.ChatViewModel.*
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll

class ChatActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val senderToken = intent.getStringExtra(SENDER_TOKEN)!!
        val addressee = intent.getStringExtra(ADDRESSEE_KEY)!!

        val chatViewModel: ChatViewModel by viewModels()
        chatViewModel.startChat(senderToken, addressee)

        val messages = mutableStateListOf<Message>()

        lifecycleScope.launchWhenCreated {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                val getMessages = async {
                    chatViewModel.getMessages().collect {
                        messages.apply {
                            clear()
                            addAll(it)
                        }
                    }
                }
                val getError = async {
                    chatViewModel.getError().collect {
                        it?.let {
                            Toast.makeText(this@ChatActivity, it, Toast.LENGTH_SHORT).show()
                            finish()
                        }
                    }
                }

                awaitAll(getMessages, getError)
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
                    MessageBar { chatViewModel.sendMessage(it) }
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
    fun ChatView(messages: List<Message>, modifier: Modifier = Modifier) =
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
        private const val SENDER_TOKEN = "sender_token"
        private const val ADDRESSEE_KEY = "addressee-key"

        fun intent(from: Context, jwt: String, addressee: String): Intent =
            Intent(from, ChatActivity::class.java)
                .apply {
                    putExtra(SENDER_TOKEN, jwt)
                    putExtra(ADDRESSEE_KEY, addressee)
                }
    }
}