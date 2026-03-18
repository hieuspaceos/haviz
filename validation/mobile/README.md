# Mobile Validation Tests

## Cần test khi build React Native app

### Push Notification
- [ ] FCM push notification (Android)
- [ ] APNs push notification (iOS)
- [ ] Background notification reliability
- [ ] Notification click → mở đúng conversation

### Approve Flow
- [ ] Nhận AI draft qua push notification
- [ ] Approve/Reject/Edit trên mobile
- [ ] Gửi tin nhắn sau approve

### Voice Note
- [ ] Ghi âm voice note 15 giây
- [ ] Upload → Groq Whisper transcribe
- [ ] AI extract structured data (sản phẩm, budget, intent)

### Real-time
- [ ] WebSocket kết nối ổn định
- [ ] Reconnect khi mất mạng
- [ ] Inbox cập nhật real-time

### Platform
- [ ] iOS 15+ (iPhone 8 trở lên)
- [ ] Android 10+ (API 29)
- [ ] Tablet layout
