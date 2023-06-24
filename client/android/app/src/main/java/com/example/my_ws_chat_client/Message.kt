package com.example.my_ws_chat_client

data class Message(val content: String, val type: MsgType)

enum class MsgType {
    ME, OTHER;
}