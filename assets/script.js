const socket = new WebSocket('ws://localhost:8087/ws');
const content = document.getElementById('content');

// Connection opened
socket.addEventListener('open', function (event) {
    socket.send('Hello Server!');

    content.innerHTML = content.innerHTML + "<i>Connection opened!</i><br>";
});

// Connection closed
socket.addEventListener('close', function (event) {
    content.innerHTML = content.innerHTML + "<i>Connection closed!</i><br>";
});

// Listen for messages
socket.addEventListener('message', function (event) {
    const msg = '&nbsp;&nbsp;-> Message from server: <b>' + event.data + '</b><br>';
    content.innerHTML = content.innerHTML + msg;
});
