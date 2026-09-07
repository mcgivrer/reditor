# language: fr
Fonctionnalité: Détection du langage par extension
  En tant qu'utilisateur de reditor
  Je veux que le langage soit détecté automatiquement selon l'extension du fichier
  Afin de bénéficier de la coloration syntaxique adaptée

  Plan du scénario: Détection du langage selon l'extension
    Étant donné un fichier nommé "<nom_de_fichier>"
    Alors le langage détecté est "<langage>"

    Exemples:
      | nom_de_fichier | langage    |
      | Main.java      | Java       |
      | app.kt         | Kotlin     |
      | lib.rs         | Rust       |
      | index.html     | HTML       |
      | style.css      | CSS        |
      | script.js      | JavaScript |
      | README.md      | Markdown   |
      | deploy.sh      | Bash       |
      | app.properties | Properties |
      | config.ini     | INI        |
      | data.json      | JSON       |
      | schema.toml    | TOML       |
      | notes.txt      | Texte      |
