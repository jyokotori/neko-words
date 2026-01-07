import typer
from typing import Optional

app = typer.Typer(
    name="nekowords",
    help="CLI for Neko Words - Vocabulary Builder",
    no_args_is_help=True,
)

from .commands.add import add_word
from .commands.review import review

app.command(name="add")(add_word)
app.command(name="review")(review)


@app.callback()
def main(
    version: bool = typer.Option(
        None,
        "--version",
        "-v",
        help="Show the application's version and exit.",
        is_eager=True,
    )
):
    if version:
        typer.echo("Neko Words CLI v0.1.0")
        raise typer.Exit()

if __name__ == "__main__":
    app()
