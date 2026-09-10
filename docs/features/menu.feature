# language: fr
Fonctionnalité: Barre de menu
  En tant qu'utilisateur de reditor
  Je veux piloter les actions courantes depuis un menu
  Afin de ne pas dépendre uniquement des raccourcis clavier

  Scénario: Ouvrir le menu avec F10
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Alors le menu "Fichier" est actif

  Scénario: Naviguer entre les menus avec les flèches
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Et j'appuie sur "Right"
    Alors le menu "Édition" est actif

  Scénario: Refermer le menu avec Échap
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Et j'appuie sur "Échap"
    Alors le menu n'est plus actif

  Scénario: Créer un nouvel onglet depuis le menu Fichier
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Et j'appuie sur "Entrée"
    Alors le nombre d'onglets ouverts est 2

  Scénario: Enregistrer sous depuis le menu propose un dialogue de sélection
    Étant donné un nouvel onglet vide
    Quand je tape "class Demo {}"
    Et j'appuie sur "F10"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors une fenêtre de dialogue "Enregistrer sous" est affichée
    Quand je saisis le nom de fichier "demo.java" dans le dialogue
    Et je confirme le dialogue de fichier
    Alors aucune fenêtre de dialogue n'est affichée
    Et le nom de l'onglet actif est "demo.java"
    Et le langage détecté est "Java"

  Scénario: Ouvrir un fichier depuis le menu propose un dialogue de sélection
    Étant donné un fichier "notes.txt" contenant "un secret bien gardé"
    Quand j'appuie sur "Ctrl+O"
    Alors une fenêtre de dialogue "Ouvrir" est affichée
    Quand je choisis "notes.txt" dans le dialogue de fichier
    Alors aucune fenêtre de dialogue n'est affichée
    Et la ligne 1 de l'éditeur contient "un secret bien gardé"

  Scénario: Échap referme le dialogue de fichier sans rien changer
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "Ctrl+O"
    Alors une fenêtre de dialogue "Ouvrir" est affichée
    Quand j'appuie sur "Échap"
    Alors aucune fenêtre de dialogue n'est affichée
    Et le nombre d'onglets ouverts est 1

  Scénario: Quitter sans modification ne demande pas de confirmation
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "Ctrl+Q"
    Alors l'application doit se terminer

  Scénario: Quitter avec des modifications non enregistrées demande confirmation
    Étant donné un nouvel onglet vide
    Quand je tape "texte non sauvegardé"
    Et j'appuie sur "Ctrl+Q"
    Alors l'application ne doit pas se terminer
    Et une invite "Modifications non enregistrées. Quitter quand même ? (o/n)" est affichée

  Scénario: Refuser de quitter conserve le travail en cours
    Étant donné un nouvel onglet vide
    Quand je tape "texte non sauvegardé"
    Et j'appuie sur "Ctrl+Q"
    Et j'appuie sur "n"
    Alors l'application ne doit pas se terminer
    Et aucune invite n'est affichée

  Scénario: Ouvrir le manuel utilisateur depuis le menu Aide
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors le nom de l'onglet actif est "HELP.md"

  Scénario: Rouvrir le manuel utilisateur ne crée pas de doublon d'onglet
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Et j'appuie sur "F10"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors le nombre d'onglets ouverts est 1
    Et le nom de l'onglet actif est "HELP.md"
