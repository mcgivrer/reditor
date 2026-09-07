# language: fr
Fonctionnalité: Extraction de la structure du fichier
  En tant qu'utilisateur de reditor
  Je veux voir la liste des symboles du fichier ouvert
  Afin de naviguer rapidement dans le code

  Scénario: Un fichier Rust expose ses fonctions et structures
    Étant donné un fichier "outil.rs" contenant "struct Point {\n}\n\nfn addition(a: i32, b: i32) -> i32 {\n    a + b\n}"
    Quand j'ouvre le fichier "outil.rs"
    Alors la structure du fichier contient "struct Point"
    Et la structure du fichier contient "fn addition"

  Scénario: Un fichier Markdown expose ses titres
    Étant donné un fichier "guide.md" contenant "# Introduction\n## Installation\ntexte"
    Quand j'ouvre le fichier "guide.md"
    Alors la structure du fichier contient "Introduction"
    Et la structure du fichier contient "Installation"

  Scénario: Un fichier INI expose ses sections
    Étant donné un fichier "conf.ini" contenant "[serveur]\nport=8080\n[client]\ntimeout=30"
    Quand j'ouvre le fichier "conf.ini"
    Alors la structure du fichier contient "[serveur]"
    Et la structure du fichier contient "[client]"

  Scénario: Un fichier sans symbole reconnu n'a pas de structure
    Étant donné un fichier "notes.txt" contenant "juste du texte"
    Quand j'ouvre le fichier "notes.txt"
    Alors la structure du fichier ne contient pas "notes"
