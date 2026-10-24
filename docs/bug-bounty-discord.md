# Discord Bug Bounty — Cẩm nang tham gia đúng luật

> Cập nhật: 2026-10-24. Nguồn chính thức: [discord.com/security](https://discord.com/security) và
> chương trình trên Bugcrowd. Đây là con đường nghiên cứu bảo mật Discord **được Discord cho phép**.

## 1. Chương trình nằm ở đâu (đính chính quan trọng)

- Discord **không còn** chạy bug bounty trên HackerOne. Từ **20/05/2025**, chương trình chuyển sang
  **Bugcrowd** và hiện ở trạng thái **private** (chỉ researcher được mời mới thấy brief đầy đủ).
- Cách vào: báo cáo một lỗ hổng có tác động cao qua email **bugbounty@discordapp.com** — report
  chất lượng sẽ nhận thư mời vào chương trình trên Bugcrowd.
- Bảng tiền thưởng theo severity (Low → Critical) nằm trong brief trên Bugcrowd; trang public
  chỉ nói thưởng "theo tính hợp lệ, mức độ nghiêm trọng và phạm vi", có thể là tiền mặt.

## 2. Scope (tóm tắt từ trang chính thức)

**Trong scope:**

- Các site/dịch vụ Discord trực tiếp vận hành (không phải bên thứ ba tích hợp qua API).
- **Discord Client Activities** do chính Discord làm: Blazing 8s, Bobble League, Bobble Bash,
  Checkers in the Park, Chess in the Park, Letter League, Land-io, Know What I Meme, Poker Night,
  Putt Party, Sketch Heads, Watch Together, Whiteboard.
- Đánh cắp token/credential của người dùng khác **bên trong một Activity** (không dùng social
  engineering) — được coi là trong scope.
- Tính năng và API của Discord.

**Ngoài scope (điểm đáng chú ý):**

- Enumeration tài khoản/email/invite, ID oracle, CSRF, clickjacking, content spoofing, open CORS.
- DoS bằng cạn kiệt tài nguyên, brute force, scanner/tự động hoá hàng loạt.
- Tự khai thác trên chính tài khoản mình, social engineering/phishing, né cảnh báo unsafe-link.
- Subdomain blog/feedback/merch/status/support; mọi thứ trên `discordsays.com`; bên thứ ba.
- **Đoạn then chốt với Astral:** *"Quest exploits that only game an individual account's
  progression"* — tức exploit quest chỉ để lợi cho một tài khoản cá nhân — **nằm ngoài scope**.
  Chỉ có khai thác quy mô lớn (ví dụ sinh key hàng loạt) mới được chấp nhận. Nói cách khác:
  **farm Orb giả cho tài khoản cá nhân không phải "nghiên cứu được Discord cho phép"** — Discord
  ghi rõ không nhận loại report này, và đây cũng là hành vi vi phạm ToS.

## 3. Luật bắt buộc

1. Chỉ test trên tài khoản/server của chính mình; không ảnh hưởng người dùng khác.
2. Cấm brute force, DoS, spam, timing attack, scanner tự động.
3. Không công bố công khai cho tới khi Discord xử lý xong; sau đó theo chính sách disclosure của
   Bugcrowd.
4. Chỉ báo cáo qua Bugcrowd; finding đã tự đăng lên Reddit/X… bị loại.
5. Report race condition phải kèm script tái hiện (Python/JS) hoặc mô tả đầy đủ tầng mạng
   (method, route, thứ tự request); report "chỉ chạy được với Burp config" bị từ chối.
6. Discord cam kết phản hồi trong ~24 giờ và không khởi kiện researcher tuân thủ luật.

## 4. Dựng môi trường test an toàn

- **Tài khoản test riêng** (email phụ), không dùng tài khoản chính; không mời tài khoản thật khác
  vào server test.
- **Server test riêng**, giữ ít thành viên; bật audit log để tự theo dõi hành động của mình.
- Với desktop client: cài bản stable + bản canary vào thư mục khác nhau; chụp network bằng proxy
  chỉ trên tài khoản test; không chạy scanner tự động chống lại production.
- Ghi lại từng bước tái hiện ngay khi làm — report thiếu repro là lý do từ chối phổ biến nhất.

## 5. Template báo cáo (tiếng Anh — ngôn ngữ của Bugcrowd)

```markdown
## Title
[Severity guess] <Component>: <one-line impact>

## Summary
2–4 sentences: what is broken, what an attacker gains, why it matters at scale.

## Affected asset
<exact domain / client build / Activity name + version tested>

## Steps to reproduce
1. ...
2. ...
3. ...
Include a minimal PoC (script/request sequence). For race conditions: attach the script.

## Impact
Concrete worst case, aligned with Discord's rules (no speculation beyond evidence).

## Supporting evidence
Screenshots, HTTP request/response pairs, build hashes. Redact other users' data.

## Remediation suggestion
Short, practical fix direction.
```

## 6. Vì sao tài liệu này nằm trong repo Astral

Astral giới hạn mình ở những gì Discord chủ động cấp giao diện (local IPC cho Rich Presence,
catalog detectable-games công khai). Nếu bạn muốn nghiên cứu sâu hơn vào client Discord, đây là
lộ trình duy nhất vừa hợp pháp vừa có thu nhập: tham gia Bugcrowd theo luật trên — phần thưởng là
tiền thật, không phải Orb.

## Nguồn

- [Discord — Our Security Bug Bounty Program](https://discord.com/security)
- [Bugcrowd](https://www.bugcrowd.com) (brief private — cần thư mời)
- [Firebounty listing cho Discord (lưu trữ lịch sử HackerOne)](https://firebounty.com/1059)
