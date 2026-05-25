import re
import typer
import httpx
from rich.console import Console
from rich import print
from rich.markup import escape
from typing import Optional, List
from ..config import settings

console = Console()

_SPLIT_RE = re.compile(r"[,\s]+")


def _split_batch(text: str) -> List[str]:
    seen = set()
    out = []
    for raw in _SPLIT_RE.split(text.strip()):
        token = raw.strip().lower()
        if not token or token in seen:
            continue
        seen.add(token)
        out.append(token)
    return out


def add_word(
    word: Optional[str] = typer.Argument(None, help="Word or phrase to add"),
    language: str = typer.Option(settings.DEFAULT_LANGUAGE, "--tag", "-t", help="Language tag"),
    batch: bool = typer.Option(False, "--batch", "-b", help="Treat input as multiple words split by whitespace/comma"),
):
    """
    Add a word or phrase to Neko Words.
    If no word provided, enters interactive mode.
    """
    if word:
        if batch:
            for w in _split_batch(word):
                _add_single_word(w, language)
        else:
            _add_single_word(word, language)
        return

    # Interactive mode
    mode = "batch" if batch else "phrase"
    console.print(f"[bold green]Entering interactive mode ({language}, {mode}).[/bold green]")
    console.print("Type a word or phrase and press Enter.")
    console.print("Commands: [cyan]/batch[/cyan] toggle batch mode, [cyan]/mode[/cyan] show mode, Ctrl+C to exit.")
    try:
        while True:
            prompt_label = "[batch] >" if batch else ">"
            line = typer.prompt(prompt_label, prompt_suffix=" ")
            line = line.strip()
            if not line:
                continue
            if line in ("/batch", "/b"):
                batch = not batch
                console.print(f"[cyan]Batch mode: {'ON' if batch else 'OFF'}[/cyan]")
                continue
            if line == "/mode":
                console.print(f"[cyan]Batch mode: {'ON' if batch else 'OFF'}[/cyan]")
                continue
            if batch:
                for w in _split_batch(line):
                    _add_single_word(w, language)
            else:
                _add_single_word(line, language)
    except typer.Abort:
        console.print("\nBye!")
    except KeyboardInterrupt:
        console.print("\nBye!")


def _add_single_word(word: str, language: str):
    url = f"{settings.API_BASE_URL}/words/"
    try:
        with console.status(f"Adding '{word}'...", spinner="dots"):
            response = httpx.post(url, json={"word": word, "language": language}, timeout=30.0)

        if response.status_code == 200:
            payload = response.json()
            data = payload["word"]
            duplicate = payload.get("duplicate", False)
            if duplicate:
                console.print(f"[yellow]↻ {data['word']}[/yellow]: already learned — review reset.")
            else:
                console.print(f"[green]✓ {data['word']}[/green]: {escape(data['translation'])}")
            console.print(f"  Example: {escape(data['examples'][0]['sentence'])}")
            console.print(f"  Translation: {escape(data['examples'][0]['translation'])}")
        else:
            console.print(f"[red]Error adding {word}: {response.text}[/red]")

    except httpx.RequestError as e:
        console.print(f"[red]Connection error: {e}[/red]")
    except KeyError as e:
        console.print(f"[red]Data error: Missing key {e} in server response[/red]")
        console.print(f"Response: {response.text}")
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
