from typing import Dict, Any
import json
from openai import AsyncOpenAI
from tenacity import retry, stop_after_attempt, wait_fixed
from loguru import logger
from ..core.config import settings


if not settings.OPENAI_API_KEY:
    raise ValueError("OPENAI_API_KEY is required")

client = AsyncOpenAI(
    api_key=settings.OPENAI_API_KEY,
    base_url=settings.OPENAI_BASE_URL,
)


@retry(stop=stop_after_attempt(3), wait=wait_fixed(2))
async def enrich_word(word: str, language: str = "en") -> Dict[str, Any]:
    logger.info(f"Enriching word: {word} ({language}) | Model: {settings.OPENAI_MODEL}")

    prompt = f"""
    You are a vocabulary assistant. Analyze the {language} word "{word}".

    Rules for word forms:
    - If the input is a conjugated verb or plural noun, set "word" to the base form (lemma).
    - For IRREGULAR forms only, append the conjugation pattern after translation, e.g., "(write-wrote-written)" or "(child-children)".
    - For REGULAR forms (add -ed, -s, -ing), do NOT mention any rule.

    Return a valid JSON object:
    {{
      "word": "base form",
      "translation": "/IPA/ Chinese translation (irregular note only if applicable)",
      "examples": [
        {{"sentence": "Example in {language}", "translation": "Chinese translation"}},
        {{"sentence": "Example in {language}", "translation": "Chinese translation"}}
      ]
    }}

    Requirements:
    - Include IPA phonetic transcription at the start of translation.
    - Provide at least 2 examples, preferably related to daily life or programming/software engineering.
    - Keep translation concise.
    """

    try:
        response = await client.chat.completions.create(
            model=settings.OPENAI_MODEL,
            messages=[{"role": "user", "content": prompt}],
            response_format={"type": "json_object"},
        )

        content = response.choices[0].message.content
        if not content:
            raise ValueError("Empty response from LLM")

        logger.info("LLM raw response for {}: {}", word, content)

        data = json.loads(content)
        logger.debug(f"LLM Response for {word}: {data}")
        return data

    except Exception as e:
        logger.error(f"Error enriching word {word}: {e}")
        raise
