# language: fr
Fonctionnalité: Support de la souris
  En tant qu'utilisateur de reditor
  Je veux piloter le menu, l'explorateur et la sélection de texte à la souris
  Afin de ne pas dépendre uniquement des raccourcis clavier

  Scénario: Ouvrir un menu en cliquant sur son titre dans la barre
    Étant donné un nouvel onglet vide
    Quand je clique sur le menu "Édition"
    Alors le menu "Édition" est actif

  Scénario: Sélectionner un fichier dans l'explorateur d'un clic
    Étant donné un fichier "notes.txt" contenant "un secret bien gardé"
    Et un dossier a été ouvert explicitement
    Quand je clique sur l'entrée "notes.txt" dans l'explorateur
    Alors le nom de l'onglet actif est "notes.txt"

  Scénario: Sélectionner du texte à la souris puis le couper et le coller
    Étant donné un fichier "mots.txt" contenant "bonjour le monde"
    Quand j'ouvre le fichier "mots.txt"
    Et je sélectionne à la souris de la colonne 0 à la colonne 7 sur la ligne 1 de l'éditeur
    Et j'appuie sur "Ctrl+X"
    Alors la ligne 1 de l'éditeur contient " le monde"
    Quand j'appuie sur "Fin"
    Et j'appuie sur "Ctrl+V"
    Alors la ligne 1 de l'éditeur contient " le mondebonjour"
