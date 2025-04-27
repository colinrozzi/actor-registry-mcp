document.addEventListener('DOMContentLoaded', () => {
    // Elements
    const statusEl = document.querySelector('.websocket-status');
    const connectBtn = document.getElementById('connect-btn');
    const sendBtn = document.getElementById('send-btn');
    const messageInput = document.getElementById('message-input');
    const messagesList = document.getElementById('messages-list');
    
    // WebSocket connection
    let socket = null;
    
    // Connect/disconnect WebSocket
    connectBtn.addEventListener('click', () => {
        if (socket && socket.readyState === WebSocket.OPEN) {
            // Disconnect
            socket.close();
        } else {
            // Connect
            const host = window.location.host;
            const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            socket = new WebSocket(`${wsProtocol}//${host}/ws`);
            
            // Connection opened
            socket.addEventListener('open', () => {
                statusEl.textContent = 'Connected';
                statusEl.classList.add('connected');
                connectBtn.textContent = 'Disconnect';
                sendBtn.disabled = false;
                messageInput.disabled = false;
                
                // Add system message
                addMessage('Connected to WebSocket server', 'system');
            });
            
            // Listen for messages
            socket.addEventListener('message', (event) => {
                let message;
                try {
                    // Try to parse JSON first
                    const data = JSON.parse(event.data);
                    message = typeof data === 'object' ? JSON.stringify(data, null, 2) : data;
                } catch (e) {
                    // If not JSON, use as is
                    message = event.data;
                }
                
                addMessage(message, 'received');
            });
            
            // Connection closed
            socket.addEventListener('close', () => {
                statusEl.textContent = 'Disconnected';
                statusEl.classList.remove('connected');
                connectBtn.textContent = 'Connect';
                sendBtn.disabled = true;
                messageInput.disabled = true;
                
                // Add system message
                addMessage('Disconnected from WebSocket server', 'system');
            });
            
            // Connection error
            socket.addEventListener('error', (error) => {
                addMessage(`Error: ${error.message}`, 'error');
                console.error('WebSocket error:', error);
            });
        }
    });
    
    // Send message
    sendBtn.addEventListener('click', sendMessage);
    messageInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            sendMessage();
        }
    });
    
    function sendMessage() {
        if (!socket || socket.readyState !== WebSocket.OPEN || !messageInput.value.trim()) {
            return;
        }
        
        const message = messageInput.value;
        socket.send(message);
        addMessage(message, 'sent');
        messageInput.value = '';
    }
    
    // Add message to the list
    function addMessage(text, type) {
        const messageEl = document.createElement('div');
        messageEl.classList.add('message', type);
        messageEl.textContent = text;
        messagesList.appendChild(messageEl);
        
        // Scroll to bottom
        messagesList.scrollTop = messagesList.scrollHeight;
    }
});
