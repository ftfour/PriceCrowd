import os
import logging
import asyncio
import json
import aiohttp
from aiogram import Bot, Dispatcher, F
from aiogram.types import Message, InlineKeyboardMarkup, InlineKeyboardButton, WebAppInfo, Update
from aiogram.filters import Command

logging.basicConfig(level=logging.INFO)

TOKEN = os.getenv("BOT_TOKEN", "")
API = os.getenv("API_URL", "https://pricecrowd.ru/api")


async def post_receipt(session: aiohttp.ClientSession, qr: str, user: str = "telegram"):
    url = f"{API}/receipts/upload"
    try:
        async with session.post(url, json={"qr": qr, "user": user, "source": "telegram"}) as resp:
            return await resp.json()
    except Exception as e:
        logging.exception("post_receipt failed: %s", e)
        return {"status": "error"}


async def main():
    if not TOKEN:
        logging.error("BOT_TOKEN is not set")
        return
    bot = Bot(TOKEN)
    dp = Dispatcher()

    @dp.message(Command("start", "scan"))
    async def cmd_scan(message: Message):
        kb = InlineKeyboardMarkup()
        kb.add(InlineKeyboardButton(text="📷 Сканировать чек", web_app=WebAppInfo(url="https://pricecrowd.ru/scan")))
        await message.answer("Сканируй QR-код чека", reply_markup=kb)

    # WebApp data handler
    @dp.message(F.web_app_data)
    async def webapp_data(message: Message):
        data = message.web_app_data.data
        logging.info("web_app_data: %s", data)
        async with aiohttp.ClientSession() as session:
            res = await post_receipt(session, data, user=str(message.from_user.id) if message.from_user else "telegram")
        await message.answer("Чек получен: " + json.dumps(res))

    # Fallback handler to accept QR pasted as text
    @dp.message()
    async def catch_all(message: Message):
        if message.text and ("t=" in message.text and "fn=" in message.text):
            async with aiohttp.ClientSession() as session:
                res = await post_receipt(session, message.text, user=str(message.from_user.id) if message.from_user else "telegram")
            await message.answer("Чек отправлен: " + json.dumps(res))

    await dp.start_polling(bot)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except (KeyboardInterrupt, SystemExit):
        logging.info("Bot stopped")

