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

  Scénario: Enregistrer sous depuis le menu renomme l'onglet
    Étant donné un nouvel onglet vide
    Quand je tape "class Demo {}"
    Et j'appuie sur "F10"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors une invite "Enregistrer sous :" est affichée
    Quand je valide l'invite avec "demo.java"
    Alors le nom de l'onglet actif est "demo.java"
    Et le langage détecté est "Java"

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
