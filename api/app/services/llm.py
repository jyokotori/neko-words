from typing import Dict, Any
import json
from openai import AsyncOpenAI, AsyncAzureOpenAI
from tenacity import retry, stop_after_attempt, wait_fixed
from loguru import logger
from ..core.config import settings


def _create_client():
    """Create OpenAI client based on configuration."""
    if settings.LLM_PROVIDER == "azure":
        if not settings.AZURE_OPENAI_API_KEY or not settings.AZURE_OPENAI_ENDPOINT:
            raise ValueError("Azure OpenAI requires AZURE_OPENAI_API_KEY and AZURE_OPENAI_ENDPOINT")
        # Strip trailing path if user provided full URL
        endpoint = settings.AZURE_OPENAI_ENDPOINT.rstrip("/")
        for suffix in ["/openai/v1", "/openai"]:
            if endpoint.endswith(suffix):
                endpoint = endpoint[:-len(suffix)]
                break
        return AsyncAzureOpenAI(
            api_key=settings.AZURE_OPENAI_API_KEY,
            azure_endpoint=endpoint,
            api_version=settings.AZURE_OPENAI_API_VERSION,
        )
    else:
        if not settings.OPENAI_API_KEY:
            raise ValueError("OpenAI requires OPENAI_API_KEY")
        return AsyncOpenAI(
            api_key=settings.OPENAI_API_KEY,
            base_url=settings.OPENAI_BASE_URL,
        )


def _get_model_name() -> str:
    """Get model name based on provider."""
    if settings.LLM_PROVIDER == "azure":
        return settings.AZURE_DEPLOYMENT_NAME
    return settings.OPENAI_MODEL


client = _create_client()

@retry(stop=stop_after_attempt(3), wait=wait_fixed(2))
async def enrich_word(word: str, language: str = "en") -> Dict[str, Any]:
    model_name = _get_model_name()
    logger.info(f"Enriching word: {word} ({language}) | Provider: {settings.LLM_PROVIDER} | Model: {model_name}")
    
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
            model=model_name,
            messages=[{"role": "user", "content": prompt}],
            response_format={"type": "json_object"}
        )
        
        content = response.choices[0].message.content
        if not content:
            raise ValueError("Empty response from LLM")
        
        # Log raw LLM output so it is visible in container logs for debugging
        logger.info("LLM raw response for {}: {}", word, content)
            
        data = json.loads(content)
        logger.debug(f"LLM Response for {word}: {data}")
        return data
        
    except Exception as e:
        logger.error(f"Error enriching word {word}: {e}")
        raise
