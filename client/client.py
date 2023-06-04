from websocket import create_connection
import threading
import json


def write_msg_blocking(ws):
    while True:
        userinput = input("<addressee, sender, msg>: ").split(",")
        encoded_smg = json.dumps(
            {'addressee': userinput[0], 'sender': userinput[1], 'msg': userinput[2]}
        )
        ws.send(encoded_smg)


def publish_msgs(ws):
    try:
        while True:
            msg = ws.recv()
            # obj = json.loads(msg)
            # print("new msg from: " + obj["name"] + ", Msg: " + obj["msg"])
            print("new msg received: " + msg)
    except:
        print("Stoping msg publishing...")


def main():
    print("Creating ws connection...")
    ws = create_connection("ws://localhost:3000/messages")
    print("Connection established!")
    writing_msgs = threading.Thread(target=write_msg_blocking, args=[ws])
    reading_msgs = threading.Thread(target=publish_msgs, args=[ws])
    writing_msgs.start()
    reading_msgs.start()

    try:
        writing_msgs.join()
        reading_msgs.join()
    except KeyboardInterrupt:
        print("Closing ws connection...")
        ws.close()
        print("Done")


if __name__ == '__main__':
    main()
