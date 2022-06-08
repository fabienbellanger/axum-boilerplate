const username = document.querySelector("#username");
const join_btn = document.querySelector("#join-chat");
const textarea = document.querySelector("#chat");
const input = document.querySelector("#input");

join_btn.addEventListener("click", function(e) {
    this.disabled = true;

    const websocket = new WebSocket("ws://localhost:8087/chat-ws");

    websocket.onopen = function() {
        console.log("connection opened");
        websocket.send(username.value);
    }

    const btn = this;

    websocket.onclose = function() {
        console.log("connection closed");
        btn.disabled = false;
    }

    websocket.onmessage = function(e) {
        console.log("received message: "+e.data);
        textarea.value += e.data+"\r\n";
    }

    input.onkeydown = function(e) {
        if (e.key == "Enter") {
            websocket.send(input.value);
            input.value = "";
        }
    }
});
