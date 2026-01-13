import typer
import httpx
from rich.console import Console
from rich import print
from rich.markup import escape
from typing import Optional
from ..config import settings

console = Console()


def add_word(
    word: Optional[str] = typer.Argument(None, help="Word or phrase to add"),
    language: str = typer.Option(settings.DEFAULT_LANGUAGE, "--tag", "-t", help="Language tag"),
):
    """
    Add a word or phrase to Neko Words.
    If no word provided, enters interactive mode.
    """
    if word:
        _add_single_word(word, language)
    else:
        # Interactive mode
        console.print(f"[bold green]Entering interactive mode ({language}).[/bold green]")
        console.print("Type a word or phrase and press Enter.")
        console.print("Press Ctrl+C to exit.")
        try:
            while True:
                line = typer.prompt(">", prompt_suffix=" ")
                if not line.strip():
                    continue
                _add_single_word(line.strip(), language)
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
            data = response.json()
            console.print(f"[green]✓ {data['word']}[/green]: {escape(data['translation'])}")
            console.print(f"  Example: {escape(data['examples'][0]['sentence'])}")
            console.print(f"  Translation: {escape(data['examples'][0]['translation'])}")
        elif response.status_code == 400:
            console.print(f"[yellow]! {word} already exists.[/yellow]")
        else:
            console.print(f"[red]Error adding {word}: {response.text}[/red]")
            
    except httpx.RequestError as e:
        console.print(f"[red]Connection error: {e}[/red]")
    except KeyError as e:
        console.print(f"[red]Data error: Missing key {e} in server response[/red]")
        console.print(f"Response: {response.text}")
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
