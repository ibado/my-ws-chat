from websocket import create_connection, WebSocketConnectionClosedException
import threading
import json


def write_msg_blocking(ws):
    try:
        addressee = input("<addressee>: ")
        init_chat = json.dumps({'addressee_nickname': addressee, 'type': 'init_chat' })
        ws.send(init_chat)
        while True:
            msg_input = input()
            message = json.dumps({'msg': msg_input, 'type': 'msg' })
            ws.send(message)
    except WebSocketConnectionClosedException as e:
        print("Stop sending msgs..." + str(e))


def publish_msgs(ws):
    try:
        while True:
            msg = ws.recv()
            print(msg)
    except WebSocketConnectionClosedException as e:
        print("Stop receiving msgs..." + str(e))


def main():
    print("Creating ws connection...")
    jwt = input("jwt: ")
    ws = create_connection("ws://localhost:7777/chat", header={"Authorization": f"Bearer {jwt}"})
    print("Connection established!")
    writing_msgs = threading.Thread(target=write_msg_blocking, args=[ws])
    reading_msgs = threading.Thread(target=publish_msgs, args=[ws])
    writing_msgs.start()
    reading_msgs.start()

    try:
        writing_msgs.join()
        reading_msgs.join()
    except KeyboardInterrupt:
        print("\nClosing ws connection...")
        ws.close()
        print("Done")


if __name__ == '__main__':
    main()
