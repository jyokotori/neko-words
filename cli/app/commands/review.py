import typer
import httpx
from rich.console import Console
from rich.prompt import Prompt
from rich.markup import escape
from rich.panel import Panel
from ..config import settings
import sys

console = Console()

# Review Grades
GRADES = {
    "1": "again",
    "2": "hard",
    "3": "good",
    "4": "easy"
}

def review(
    language: str = typer.Option(settings.DEFAULT_LANGUAGE, "--tag", "-t", help="Language tag"),
    limit: int = typer.Option(50, help="Max words to review"),
):
    """
    Start a review session.
    """
    try:
        # Fetch reviews
        url = f"{settings.API_BASE_URL}/reviews/due"
        response = httpx.get(url, params={"limit": limit, "language": language}, timeout=10.0)
        
        if response.status_code != 200:
             console.print(f"[red]Error fetching reviews: {response.text}[/red]")
             return
             
        items = response.json()
        if not items:
            console.print("[green]No words to review! 🎉[/green]")
            return
            
        console.print(f"[bold]Starting review for {len(items)} words...[/bold]")
        console.print("Controls: Space (Next/Reveal), 1-4 (Grade), Enter (Skip), Ctrl+C (Exit)")
        
        for item in items:
            _review_loop(item)
            
    except httpx.ConnectError:
        console.print("[red]Could not connect to backend.[/red]")
    except KeyboardInterrupt:
        console.print("\n[yellow]Review paused.[/yellow]")

def _review_loop(item: dict):
    word_data = item['word']
    word_id = word_data['id']
    
    # Clear screen/separator
    console.rule()
    
    # Step 1: Show Word + Example
    console.print(Panel(f"[bold cyan]{word_data['word']}[/bold cyan]", title="Review"))
    
    if word_data['examples']:
        ex = word_data['examples'][0]
        # Use escape to prevent markup errors in content
        console.print(f"[italic]{escape(ex['sentence'])}[/italic]")
    
    # Step 2: Wait for reveal
    typer.prompt("", default="", show_default=False, prompt_suffix="Press Enter to show answer...")
    
    # Step 3: Show Answer
    console.print(f"\n[bold green]{escape(word_data['translation'])}[/bold green]")
    
    # Show detail examples
    for ex in word_data['examples']:
        console.print(f"- {escape(ex['sentence'])}")
        console.print(f"  [dim]{escape(ex['translation'])}[/dim]")
        
    # Step 4: Grade
    console.print("\n[dim]1:Again  2:Hard  3:Good(Default)  4:Easy[/dim]")
    
    while True:
        grade_input = typer.prompt("Grade", default="3", show_default=False)
        
        if not grade_input:
             grade_input = "3"
             
        if grade_input in GRADES:
            grade = GRADES[grade_input]
            _submit_review(word_id, grade)
            break
        else:
            console.print("[red]Invalid grade. Please enter 1-4.[/red]")

def _submit_review(word_id: str, grade: str):
    url = f"{settings.API_BASE_URL}/reviews/{word_id}/log"
    try:
        httpx.post(url, json={"grade": grade})
    except Exception as e:
        console.print(f"[red]Failed to submit review: {e}[/red]")
