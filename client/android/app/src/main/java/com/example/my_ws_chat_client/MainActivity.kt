package com.example.my_ws_chat_client

import android.Manifest.permission.POST_NOTIFICATIONS
import android.app.AlertDialog
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager.PERMISSION_GRANTED
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts.RequestPermission
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.app.ActivityCompat.checkSelfPermission
import com.example.my_ws_chat_client.chat.ChatActivity
import com.example.my_ws_chat_client.notifications.NotificationsService
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme

class MainActivity : ComponentActivity() {

    private val recentChats: SnapshotStateList<String> = mutableStateListOf()

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val jwt = intent.getStringExtra(JWT_KEY)!!

        setContent {
            MywschatclientTheme {
                Surface(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(10.dp),
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
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(5.dp),
                            label = { Text(text = "Addressee nickname") },
                        )
                        Button(
                            onClick = { startChatActivity(jwt, addresseeValue.text) },
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(5.dp),
                        ) {
                            Text(text = "Start Chat")
                        }
                        if (recentChats.isNotEmpty()) {
                            RecentChats(jwt)
                        }
                    }
                }
            }
        }
    }

    @OptIn(ExperimentalFoundationApi::class)
    @Composable
    private fun RecentChats(jwt: String) {
        LazyColumn(
            Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            item { Text(text = "Recent chats", Modifier.padding(10.dp)) }
            items(recentChats.size) { idx ->
                Text(
                    style = TextStyle(
                        textAlign = TextAlign.Center,
                        fontSize = 16.sp,
                        color = Color.DarkGray
                    ),
                    modifier = Modifier
                        .combinedClickable(
                            onClick = {
                                startChatActivity(
                                    jwt,
                                    recentChats[idx]
                                )
                            },
                            onLongClick = {
                                val addressee = recentChats[idx]
                                AlertDialog
                                    .Builder(this@MainActivity)
                                    .setTitle("Hello :D")
                                    .setMessage("You want to delete $addressee from recent chats?")
                                    .setPositiveButton(
                                        "Delete"
                                    ) { _, _ -> removeAddressee(addressee) }
                                    .show()

                            }
                        )
                        .fillMaxWidth()
                        .padding(10.dp),
                    text = AnnotatedString(recentChats[idx]),
                )
            }
        }
    }

    override fun onStart() {
        super.onStart()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (checkSelfPermission(this, POST_NOTIFICATIONS) == PERMISSION_GRANTED) {
                startNotificationService()
            } else {
                registerForActivityResult(RequestPermission()) { isGranted ->
                    if (isGranted) startNotificationService()
                    else {
                        showToast("You won't get notifications for new messages!")
                        showToast("For enable it go to Settings -> Applications -> Permissions")
                    }
                }.launch(POST_NOTIFICATIONS)
            }
        } else {
            startNotificationService()
        }
    }

    override fun onResume() {
        super.onResume()
        recentChats.clear()
        recentChats.addAll(getRecentChats())
    }

    private fun removeAddressee(addressee: String) {
        removeAddresseeFromRecentChats(addressee)
        recentChats.remove(addressee)
    }

    private fun startChatActivity(jwt: String, addressee: String) {
        val intent = ChatActivity.intent(this@MainActivity, jwt, addressee)
        startActivity(intent)
    }

    private fun startNotificationService() {
        Intent(this@MainActivity, NotificationsService::class.java)
            .let(::startForegroundService)
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