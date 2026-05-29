from datetime import datetime, timezone

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth import get_current_user
from app.database import async_session
from app.models import Note, User
from app.schemas import NoteCreate, NoteOut, NoteUpdate, SyncRequest, SyncResponse

router = APIRouter(prefix="/api/notes", tags=["notes"])


async def get_db():
    async with async_session() as session:
        yield session


@router.get("", response_model=list[NoteOut])
async def list_notes(user: User = Depends(get_current_user)):
    async with async_session() as session:
        result = await session.execute(
            select(Note).where(Note.user_id == user.id).order_by(Note.updated_at.desc())
        )
        return [NoteOut.model_validate(n) for n in result.scalars().all()]


@router.post("", response_model=NoteOut, status_code=status.HTTP_201_CREATED)
async def create_note(body: NoteCreate, user: User = Depends(get_current_user)):
    async with async_session() as session:
        note = Note(user_id=user.id, title=body.title, content=body.content)
        session.add(note)
        await session.commit()
        await session.refresh(note)
        return NoteOut.model_validate(note)


@router.get("/{note_id}", response_model=NoteOut)
async def get_note(note_id: str, user: User = Depends(get_current_user)):
    async with async_session() as session:
        result = await session.execute(
            select(Note).where(Note.id == note_id, Note.user_id == user.id)
        )
        note = result.scalar_one_or_none()
        if not note:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)
        return NoteOut.model_validate(note)


@router.put("/{note_id}", response_model=NoteOut)
async def update_note(note_id: str, body: NoteUpdate, user: User = Depends(get_current_user)):
    async with async_session() as session:
        result = await session.execute(
            select(Note).where(Note.id == note_id, Note.user_id == user.id)
        )
        note = result.scalar_one_or_none()
        if not note:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

        if body.title is not None:
            note.title = body.title
        if body.content is not None:
            note.content = body.content
        note.updated_at = datetime.now(timezone.utc)

        await session.commit()
        await session.refresh(note)
        return NoteOut.model_validate(note)


@router.delete("/{note_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_note(note_id: str, user: User = Depends(get_current_user)):
    async with async_session() as session:
        result = await session.execute(
            select(Note).where(Note.id == note_id, Note.user_id == user.id)
        )
        note = result.scalar_one_or_none()
        if not note:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

        await session.delete(note)
        await session.commit()


@router.post("/sync", response_model=SyncResponse)
async def sync_notes(body: SyncRequest, user: User = Depends(get_current_user)):
    async with async_session() as session:
        now = datetime.now(timezone.utc)

        for change in body.changes:
            if change.deleted:
                result = await session.execute(
                    select(Note).where(Note.id == change.id, Note.user_id == user.id)
                )
                note = result.scalar_one_or_none()
                if note:
                    await session.delete(note)
            else:
                result = await session.execute(
                    select(Note).where(Note.id == change.id, Note.user_id == user.id)
                )
                note = result.scalar_one_or_none()

                if note:
                    if change.updated_at > note.updated_at.replace(tzinfo=timezone.utc):
                        note.title = change.title
                        note.content = change.content
                        note.updated_at = change.updated_at
                else:
                    note = Note(
                        id=change.id,
                        user_id=user.id,
                        title=change.title,
                        content=change.content,
                        created_at=change.updated_at,
                        updated_at=change.updated_at,
                    )
                    session.add(note)

        await session.commit()

        result = await session.execute(
            select(Note).where(Note.user_id == user.id).order_by(Note.updated_at.desc())
        )
        notes = [NoteOut.model_validate(n) for n in result.scalars().all()]

        return SyncResponse(notes=notes, server_time=now)
