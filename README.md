# 📝 Noter

> A desktop note-taking application built with **Tauri, Rust, SvelteKit, and FastAPI**.
>
> My goal with this project wasn't just to build a notes app—it was to learn how Tauri works, understand Rust in a practical setting, and see how a modern desktop application can combine web technologies with native performance.

---

## Why I Built This

I had always been curious about Rust, but most learning resources jumped straight into ownership, lifetimes, and advanced concepts.

Tauri gave me a more approachable path.

Instead of learning Rust in isolation, I learned it while building something useful:

- A desktop application
- Local SQLite storage
- Authentication
- Offline-first notes
- Cloud synchronization
- Native installers

This project became my hands-on introduction to Rust and desktop development.

You can read more about that experience here:

[**My First Day with Tauri: Why It Made Rust Feel Approachable**](https://medium.com/@Akio08/my-first-day-with-tauri-why-it-made-rust-feel-approachable-73f985d6ee69)

---

## ✨ Features

### Notes Management

- Create notes
- Edit notes
- Delete notes
- Persistent local storage

### Authentication

- User registration
- User login
- JWT-based authentication

### Offline First

- Notes are stored locally in SQLite
- Works without internet access
- Changes are tracked until synchronization

### Synchronization

- Manual sync with backend
- Conflict handling using timestamps
- Cross-session persistence

### Native Desktop Experience

- Built with Tauri
- Small binary size
- Native installers
- No Electron

---

## 🏗️ Architecture

```text
┌─────────────────────┐
│     SvelteKit UI     │
└──────────┬──────────┘
           │ Tauri IPC
           ▼
┌─────────────────────┐
│      Rust Core       │
│   Commands + State   │
└───────┬───────┬─────┘
        │       │
        ▼       ▼
  SQLite DB   FastAPI
   (Local)    Backend
```

### Frontend

- SvelteKit
- TypeScript
- Svelte Stores

### Desktop Layer

- Tauri v2
- Rust
- SQLite (rusqlite)

### Backend

- FastAPI
- SQLAlchemy
- JWT Authentication
- SQLite

---

## 🛠 Tech Stack

### Frontend

- SvelteKit
- Svelte 5
- TypeScript
- Vite

### Desktop

- Tauri v2
- Rust

### Backend

- FastAPI
- SQLAlchemy
- SQLite
- JWT
- bcrypt

---

## 📂 Project Structure

```text
.
├── backend/            # FastAPI backend
├── src/                # SvelteKit frontend
├── src-tauri/          # Rust/Tauri application
├── ARCHITECTURE.md     # Detailed architecture explanation
├── SDLC.md             # Development process documentation
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

- Node.js
- Rust
- Cargo
- Tauri CLI
- Python 3.11+

### Clone Repository

```bash
git clone https://github.com/aki-008/noter.git

cd noter
```

### Install Dependencies

```bash
npm install
```

### Run Backend

```bash
cd backend

pip install -r requirements.txt

uvicorn main:app --reload
```

### Run Desktop App

```bash
npm run tauri dev
```

---

## 📦 Production Build

Build the desktop application:

```bash
npm run tauri build
```

Generated installers can be found inside:

```text
src-tauri/target/release/bundle/
```

---

## 📚 What I Learned

This project taught me:

### Rust Fundamentals

- Structs
- Enums
- Traits
- Error handling with `Result`
- Async programming
- Shared state with `Mutex`

### Tauri Concepts

- IPC communication
- Commands
- Application state
- Resource bundling
- Native packaging

### Desktop Application Design

- Frontend ↔ Rust communication
- Rust ↔ Backend communication
- Offline-first architecture
- Synchronization workflows

Most importantly, it showed me that Rust becomes much easier to understand when you're solving a real problem instead of studying isolated examples.

---

## 📖 Documentation

Additional documentation is available in the repository:

- `ARCHITECTURE.md` — Detailed explanation of every layer and file.
- `SDLC.md` — Development journey, decisions, and implementation details.

---

## 🔮 Future Improvements

- Full-text search
- Markdown support
- Rich text editor
- Dark mode
- Tags and categories
- Cross-device synchronization
- Linux support
- macOS support

---

## 📸 Screenshots

<img width="1919" height="1079" alt="login" src="https://github.com/user-attachments/assets/fa1a5614-27c7-4286-a08e-20d5af0e52f4" />
<img width="1919" height="1079" alt="notes" src="https://github.com/user-attachments/assets/27fd6d2a-7f8b-4a83-9417-f20f0ce29574" />


```text
screenshots/
├── login.png
├── notes.png
```

---

## 🤝 Contributing

Contributions, suggestions, and feedback are welcome.

Feel free to open an issue or submit a pull request.

---

## 📄 License

MIT License

---

## 👨‍💻 Author

**Shaswat Singh**

- GitHub: https://github.com/aki-008
- LinkedIn: https://www.linkedin.com/in/shaswat-singh-594648265/
- Portfolio: https://shaswatsinghsite.vercel.app/
---

> "Tauri didn't just help me build a desktop app—it made Rust feel approachable enough that I actually wanted to keep learning it."
