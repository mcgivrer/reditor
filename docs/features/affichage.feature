# language: fr
Fonctionnalité: Affichage des panneaux
  En tant qu'utilisateur de reditor
  Je veux afficher ou masquer l'explorateur et la structure
  Afin d'adapter l'interface à mon contexte de travail

  Scénario: L'explorateur est masqué sans dossier explicitement ouvert
    Étant donné aucun dossier n'a été ouvert explicitement
    Alors le panneau "Explorateur" est caché

  Scénario: L'explorateur est visible quand un dossier est ouvert
    Étant donné un dossier a été ouvert explicitement
    Alors le panneau "Explorateur" est visible

  Scénario: Ctrl+B bascule la visibilité de l'explorateur
    Étant donné aucun dossier n'a été ouvert explicitement
    Quand j'appuie sur "Ctrl+B"
    Alors le panneau "Explorateur" est visible
    Quand j'appuie sur "Ctrl+B"
    Alors le panneau "Explorateur" est caché

  Scénario: La structure est visible par défaut
    Étant donné un nouvel onglet vide
    Alors le panneau "Structure" est visible

  Scénario: Le menu Affichage bascule la structure
    Étant donné un nouvel onglet vide
    Quand j'appuie sur "F10"
    Et j'appuie sur "Right"
    Et j'appuie sur "Right"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Alors le panneau "Structure" est caché
