# language: fr
Fonctionnalité: Ouverture d'un dossier
  En tant qu'utilisateur de reditor
  Je veux ouvrir n'importe quel dossier du système de fichiers depuis le menu
  Afin de changer de projet ouvert sans relancer l'application

  Scénario: Ouvrir un dossier depuis le menu Fichier affiche un dialogue de sélection
    Étant donné aucun dossier n'a été ouvert explicitement
    Quand j'appuie sur "F10"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors une fenêtre de dialogue "Ouvrir un dossier" est affichée

  Scénario: Ouvrir un dossier avec le raccourci clavier Ctrl+Maj+O
    Étant donné aucun dossier n'a été ouvert explicitement
    Quand j'appuie sur "Ctrl+Maj+O"
    Alors une fenêtre de dialogue "Ouvrir un dossier" est affichée

  Scénario: Échap referme le dialogue d'ouverture de dossier sans rien changer
    Étant donné aucun dossier n'a été ouvert explicitement
    Quand j'appuie sur "F10"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Et j'appuie sur "Échap"
    Alors aucune fenêtre de dialogue n'est affichée
    Et le panneau "Explorateur" est caché

  Scénario: Descendre puis remonter au-delà du point de départ pour choisir un dossier
    Étant donné un dossier "sous-dossier" existe
    Et aucun dossier n'a été ouvert explicitement
    Quand j'appuie sur "F10"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors une fenêtre de dialogue "Ouvrir un dossier" est affichée
    Quand j'appuie sur "Bas"
    Et j'appuie sur "Droite"
    Et j'appuie sur "Gauche"
    Et j'appuie sur "Gauche"
    Et j'appuie sur "Gauche"
    Et j'appuie sur "Gauche"
    Et j'appuie sur "Gauche"
    Et je confirme le dialogue de fichier
    Alors aucune fenêtre de dialogue n'est affichée
    Et le dossier racine de l'explorateur est le dossier parent du dossier de travail
    Et le panneau "Explorateur" est visible
