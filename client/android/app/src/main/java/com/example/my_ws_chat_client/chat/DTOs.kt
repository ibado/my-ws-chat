package com.example.my_ws_chat_client.chat

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
sealed interface Response {
    @Serializable
    @SerialName("chat_init_success")
    object ChatInitSuccess : Response
    @Serializable
    @SerialName("chat_init_failure")
    data class ChatInitFailure(val error: String) : Response
    @Serializable
    @SerialName("msg")
    data class Msg(val msg: String, @SerialName("is_sender") val isSender: Boolean) : Response
}

@Serializable
sealed interface Request {
    @Serializable
    @SerialName("init_chat")
    data class InitChat(@SerialName("addressee_nickname") val addresseeNickName: String) : Request
    @Serializable
    @SerialName("msg")
    data class Msg(val msg: String) : Request
}