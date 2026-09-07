# language: fr
Fonctionnalité: Édition de texte
  En tant qu'utilisateur de reditor
  Je veux écrire, modifier et sauvegarder du texte
  Afin de gérer mes fichiers

  Scénario: Taper du texte dans un nouvel onglet
    Étant donné un nouvel onglet vide
    Quand je tape "Bonjour le monde"
    Alors la ligne 1 de l'éditeur contient "Bonjour le monde"
    Et l'onglet est marqué comme modifié

  Scénario: Un retour à la ligne crée une nouvelle ligne
    Étant donné un nouvel onglet vide
    Quand je tape "premiere\nseconde"
    Alors la ligne 1 de l'éditeur contient "premiere"
    Et la ligne 2 de l'éditeur contient "seconde"

  Scénario: Sauvegarder un fichier modifié
    Étant donné un fichier "notes.txt" contenant "ancien contenu"
    Quand j'ouvre le fichier "notes.txt"
    Et j'appuie sur "Fin"
    Et je tape " ajouté"
    Et j'appuie sur "Ctrl+S"
    Alors le fichier "notes.txt" contient sur le disque "ancien contenu ajouté"
    Et l'onglet n'est plus marqué comme modifié

  Scénario: Couper puis coller une ligne
    Étant donné un fichier "lignes.txt" contenant "alpha\nbeta\ngamma"
    Quand j'ouvre le fichier "lignes.txt"
    Et j'appuie sur "Ctrl+X"
    Alors la ligne 1 de l'éditeur contient "beta"
    Quand j'appuie sur "Ctrl+V"
    Alors la ligne 1 de l'éditeur contient "beta"
    Et la ligne 2 de l'éditeur contient "alpha"
    Et la ligne 3 de l'éditeur contient "gamma"
