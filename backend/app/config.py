import os
import uuid

DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "sqlite+aiosqlite:///./note_taker.db",
)

JWT_SECRET = os.getenv("JWT_SECRET") or str(uuid.uuid4())

JWT_ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 60 * 24
