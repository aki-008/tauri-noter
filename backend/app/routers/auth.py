from fastapi import APIRouter, HTTPException, status
from sqlalchemy import select

from app.auth import create_token, get_current_user, hash_password, verify_password
from app.database import async_session
from app.models import User
from app.schemas import LoginRequest, Token, UserCreate, UserOut

router = APIRouter(prefix="/api/auth", tags=["auth"])


@router.post("/register", response_model=Token, status_code=status.HTTP_201_CREATED)
async def register(body: UserCreate):
    async with async_session() as session:
        result = await session.execute(select(User).where(User.username == body.username))
        if result.scalar_one_or_none():
            raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Username taken")

        user = User(username=body.username, hashed_password=hash_password(body.password))
        session.add(user)
        await session.commit()
        await session.refresh(user)

        token = create_token(user.id)
        return Token(access_token=token, user=UserOut.model_validate(user))


@router.post("/login", response_model=Token)
async def login(body: LoginRequest):
    async with async_session() as session:
        result = await session.execute(select(User).where(User.username == body.username))
        user = result.scalar_one_or_none()

        if not user or not verify_password(body.password, user.hashed_password):
            raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid credentials")

        token = create_token(user.id)
        return Token(access_token=token, user=UserOut.model_validate(user))
