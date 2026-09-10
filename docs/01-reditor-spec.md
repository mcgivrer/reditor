# Spécification fonctionnelle — reditor

| | |
|---|---|
| **Produit** | reditor |
| **Type** | Éditeur de texte façon IDE, exécuté dans le terminal |
| **Statut** | En développement actif |
| **Date** | 2026-09-08 |

## 1. Introduction

### 1.1 Contexte et objectif

`reditor` est un éditeur de texte en mode terminal (TUI — *Text User
Interface*) qui reprend l'organisation visuelle d'un IDE moderne : un
explorateur de fichiers sur le côté gauche, une zone d'édition centrale à
onglets, un panneau de structure du fichier sur la droite, une barre de menu
et une barre de statut. Il vise à offrir un confort d'édition proche d'un
éditeur graphique tout en restant utilisable dans un simple terminal, sans
souris et sans serveur graphique.

### 1.2 Public visé

Toute personne éditant du code ou du texte directement depuis un terminal
(connexion SSH, environnement sans interface graphique, préférence pour le
clavier) et souhaitant un éditeur plus visuel qu'un éditeur en ligne de
commande classique, sans toutefois nécessiter la complexité d'un IDE complet.

## 2. Vue d'ensemble de l'interface

L'écran est structuré en quatre zones fixes :

```
┌ Fichier  Édition  Affichage  Aide ───────────────────────────────────────┐  ← barre de menu (toujours visible)
├ Explorateur ──────────┬ [onglet1] [onglet2] ───────────┬ Structure ─────┤
│ ▾ mon-projet           │  1  fn main() {               │ fn main        │
│   ▸ src                │  2      println!("bonjour");  │                │
│     Cargo.toml         │  3  }                          │                │
│                        │                                │                │
├────────────────────────┴────────────────────────────────┴────────────────┤
│ Rust — main.rs  Ln 1, Col 1        F10 menu  F1 aide  Ctrl+S sauver ...   │  ← barre de statut
└────────────────────────────────────────────────────────────────────────────┘
```

- **Barre de menu** : toujours affichée sur la première ligne du terminal.
- **Explorateur** (gauche, optionnel) : arborescence du dossier de travail.
- **Éditeur** (centre) : onglets de fichiers ouverts + zone de texte.
- **Structure** (droite, optionnel) : liste des symboles du fichier actif.
- **Barre de statut** (bas) : langage détecté, nom de fichier, position du
  curseur, aide contextuelle ou dernier message.

Les panneaux Explorateur et Structure se masquent automatiquement si la
largeur du terminal est insuffisante pour les afficher lisiblement (voir
§4.6), et peuvent aussi être masqués/affichés manuellement.

## 3. Fonctionnalités

### 3.1 Barre de menu

Une barre de menu façon logiciel de bureau, activable par **F10** :

| Menu | Entrée | Raccourci direct | Action |
|---|---|---|---|
| Fichier | Nouveau | Ctrl+N | Crée un nouvel onglet vide (« sans titre ») |
| Fichier | Ouvrir... | Ctrl+O | Ouvre un dialogue de sélection d'un fichier dans l'arborescence |
| Fichier | Ouvrir un dossier... | — | Ouvre un dialogue de sélection d'un dossier (navigation libre, y compris au-delà du dossier actuellement ouvert) et le fait devenir la nouvelle racine de l'explorateur |
| Fichier | Enregistrer | Ctrl+S | Sauvegarde l'onglet actif sur son fichier |
| Fichier | Enregistrer sous... | — | Ouvre une invite pour choisir un nouveau chemin ; le langage est redétecté selon la nouvelle extension |
| Fichier | Fermer l'onglet | Ctrl+W | Ferme l'onglet actif |
| Fichier | Quitter | Ctrl+Q | Quitte l'application (avec confirmation si des modifications ne sont pas enregistrées) |
| Édition | Couper la ligne | Ctrl+X | Retire la ligne courante et la place dans le presse-papiers interne |
| Édition | Copier la ligne | Ctrl+C | Copie la ligne courante dans le presse-papiers interne |
| Édition | Coller | Ctrl+V | Insère le contenu du presse-papiers comme nouvelle ligne sous le curseur |
| Édition | Aller à la ligne... | Ctrl+G | Ouvre une invite pour saisir un numéro de ligne |
| Affichage | Explorateur | Ctrl+B | Bascule l'affichage du panneau Explorateur (case à cocher reflétant l'état) |
| Affichage | Structure | — | Bascule l'affichage du panneau Structure (case à cocher reflétant l'état) |
| Compiler *(si projet compilable)* | Compiler le projet | F5 | Compile le projet avec le JDK sélectionné |
| Compiler *(si projet compilable)* | Configurer les JDK... | F6 | Ouvre le dialogue de sélection du JDK à utiliser |
| Aide | À propos | F1 | Affiche une fenêtre de rappel des raccourcis |

Navigation dans le menu : flèches gauche/droite pour changer de menu,
haut/bas pour changer d'entrée (les séparateurs sont ignorés), Entrée pour
valider, Échap ou F10 pour refermer sans agir.

### 3.2 Explorateur de fichiers

- Affiche l'arborescence du dossier de travail sous forme d'arbre indenté,
  avec une icône ▾/▸ pour les dossiers (déplié/replié) selon leur état.
- Navigation : ↑/↓ (ou j/k) pour changer de sélection, → / Entrée / l pour
  ouvrir un fichier ou déplier/replier un dossier, ← / h pour replier ou
  remonter au dossier parent.
- Ouvrir un fichier depuis l'explorateur l'ouvre dans un nouvel onglet (ou
  active l'onglet existant si le fichier est déjà ouvert) et redonne le
  focus à l'éditeur.
- Accès au panneau : **F2** (focus) ou **Ctrl+E**. Retour à l'éditeur :
  **Échap** ou **F3**.
- **Fichier > Ouvrir un dossier...** ouvre un dialogue de sélection de
  dossier, du même type que celui des dialogues « Ouvrir... » et
  « Enregistrer sous... » : ↑/↓ pour naviguer, → pour déplier un
  sous-dossier, ← pour replier ou remonter (y compris, une fois la racine
  du dialogue repliée, au-delà de son point de départ, jusqu'à la racine du
  système de fichiers), Entrée pour choisir le dossier ciblé. Ce dossier
  devient alors la nouvelle racine de l'explorateur, qui est rendu visible
  s'il ne l'était pas déjà.

### 3.3 Éditeur de texte et onglets

- Chaque fichier ouvert occupe son propre onglet ; le titre affiche le nom
  du fichier et un `*` si des modifications ne sont pas enregistrées.
- Édition standard : insertion de caractères, retour à la ligne (Entrée),
  suppression (Retour arrière / Suppr), tabulation (insère 4 espaces),
  déplacement du curseur (flèches, Origine, Fin, Page précédente/suivante).
- Numéros de ligne affichés en marge gauche de l'éditeur.
- Défilement horizontal et vertical automatique pour garder le curseur
  visible.
- Un onglet sans nom de fichier s'affiche « sans titre » ; sa sauvegarde
  passe par « Enregistrer sous... ».
- Basculer entre onglets : **Ctrl+←** / **Ctrl+→**.
- Accès au panneau éditeur : **F3**.

### 3.4 Coloration syntaxique

Le langage d'un fichier est détecté automatiquement d'après son extension
(ou, à défaut, son nom exact pour certains fichiers sans extension comme
`Makefile` ou `Dockerfile`). Chaque ligne visible est ensuite mise en
couleur (mots-clés, chaînes, nombres, commentaires, fonctions, balises,
attributs, sections, etc.) selon des règles propres au langage détecté.

Langages explicitement pris en charge : **Rust, Java, Kotlin, JavaScript,
TypeScript, Bash, Markdown, HTML, CSS, Properties, INI**, ainsi que
**JSON, TOML, YAML, Python, C, C++, Go, XML** en complément. Tout autre
fichier est traité comme du texte brut, sans coloration.

Le langage détecté est affiché en début de barre de statut, et se met à
jour automatiquement après un « Enregistrer sous... » vers une nouvelle
extension.

### 3.5 Structure du fichier (panneau « Structure »)

Liste, pour le fichier actif, les symboles significatifs qu'il contient,
avec indentation selon leur profondeur logique :

| Langage | Symboles listés |
|---|---|
| Rust | `fn`, `struct`, `enum`, `trait`, `impl`, `mod` |
| Java / Kotlin | `class`, `interface`, `enum`, `object`, `fun` |
| JavaScript / TypeScript | classes, fonctions, constantes assignées à une fonction fléchée |
| Python | `def`, `class` (indentation selon le niveau d'indentation Python) |
| Go / C / C++ | `func`, `struct`, `class`, `interface`, `type` |
| Markdown | titres `#`…`######`, indentés selon leur niveau |
| Bash | fonctions déclarées (`function nom` ou `nom()`) |
| CSS | sélecteurs de règles |
| HTML / XML | balises `h1`–`h6`, `script`, `style`, `body`, `head`, éléments avec un `id` |
| INI / TOML | sections `[section]` |
| Properties | chaque clé déclarée |
| YAML | clés de premier niveau, indentées selon leur profondeur |

Navigation : ↑/↓ pour sélectionner un symbole, Entrée pour placer le
curseur de l'éditeur sur la ligne correspondante. Accès : **F4** ou
**Ctrl+L** ; retour à l'éditeur : **Échap** ou **F3**.

### 3.6 Affichage des panneaux

- L'explorateur est **affiché par défaut** uniquement si un dossier a été
  explicitement ouvert au lancement (argument en ligne de commande pointant
  vers un dossier). Dans les autres cas (fichier isolé ouvert, ou aucun
  argument), il démarre **masqué**, mais reste accessible à tout moment via
  **Ctrl+B** ou le menu Affichage.
- Le panneau Structure est affiché par défaut.
- Chaque panneau se masque aussi automatiquement si le terminal est trop
  étroit pour l'afficher proprement (seuils internes), indépendamment de la
  préférence de l'utilisateur, qui est réappliquée dès que la place
  suffit à nouveau.

### 3.7 Barre de statut

Affiche en permanence : le langage détecté, le nom du fichier actif (et son
état modifié), la position du curseur (ligne, colonne), puis soit une aide
contextuelle sur les raccourcis principaux, soit le dernier message
d'information ou d'erreur (par exemple une confirmation de sauvegarde).

### 3.8 Compilation

Le dossier ouvert est analysé au démarrage par une série de **modules de
compilation**, chacun spécifique à un type de projet. Un premier module
prend en charge **Java** : il détecte le projet dès qu'un fichier `*.java`
existe n'importe où dans l'arborescence.

- Le menu **Compiler** (et ses raccourcis F5/F6) n'apparaît que si un
  module de compilation a détecté le projet ouvert ; il est totalement
  absent dans le cas contraire.
- **Configurer les JDK... (F6)** ouvre un dialogue listant les JDK détectés
  automatiquement, dans cet ordre de priorité : les candidats gérés par
  [sdkman](https://sdkman.io/) (`~/.sdkman/candidates/java`), la variable
  d'environnement `JAVA_HOME`, puis un exécutable `javac` trouvable dans le
  `PATH`. Un même JDK détecté par plusieurs sources n'apparaît qu'une fois.
  Si aucun JDK n'est trouvé, le dialogue l'indique explicitement. ↑/↓ pour
  choisir, Entrée pour valider, Échap pour annuler.
- **Compiler le projet (F5)** compile le projet avec le JDK sélectionné (ou,
  à défaut de sélection explicite, le premier JDK détecté). Le résultat
  (succès ou détail de l'échec, sortie de `javac`) s'affiche dans une
  fenêtre dédiée, sans quitter `reditor`.

## 4. Raccourcis clavier — récapitulatif

| Raccourci | Action |
|---|---|
| F10 | Ouvrir/fermer la barre de menu |
| F1 | Afficher/fermer la fenêtre « À propos » |
| F2 | Donner le focus à l'explorateur |
| F3 | Donner le focus à l'éditeur |
| F4 | Donner le focus à la structure |
| F5 | Compiler le projet *(si un module de compilation a détecté le projet)* |
| F6 | Configurer les JDK utilisés pour la compilation *(idem)* |
| Ctrl+N | Nouveau fichier |
| Ctrl+O | Ouvrir un fichier (dialogue de sélection) |
| Ctrl+S | Enregistrer |
| Ctrl+W | Fermer l'onglet actif |
| Ctrl+Q | Quitter (avec confirmation si modifications non enregistrées) |
| Ctrl+B | Basculer l'affichage de l'explorateur |
| Ctrl+G | Aller à une ligne donnée |
| Ctrl+X / Ctrl+C / Ctrl+V | Couper / copier / coller la ligne courante |
| Ctrl+E | Donner le focus à l'explorateur |
| Ctrl+L | Donner le focus à la structure |
| Ctrl+← / Ctrl+→ | Onglet précédent / suivant |
| Échap | Fermer un menu/une invite, ou revenir à l'éditeur depuis un panneau |

## 5. Scénarios d'utilisation typiques

1. **Ouvrir un projet et naviguer dans ses fichiers** : lancer
   `reditor mon-projet/`, l'explorateur est visible par défaut ; naviguer
   avec les flèches, ouvrir un fichier avec Entrée.
2. **Éditer rapidement un fichier isolé** : lancer `reditor notes.txt`,
   l'explorateur reste masqué pour laisser toute la place à l'édition ;
   sauvegarder avec Ctrl+S.
3. **Naviguer dans un gros fichier** : ouvrir le panneau Structure (F4),
   sélectionner une fonction, Entrée pour y sauter directement.
4. **Renommer un fichier en changeant son type** : « Enregistrer sous... »
   vers une nouvelle extension ; la coloration et la structure se mettent
   à jour immédiatement.
5. **Quitter en toute sécurité** : Ctrl+Q déclenche une confirmation si un
   onglet contient des modifications non enregistrées.

## 6. Limitations connues

- Pas de prise en charge de la souris : toute la navigation se fait au
  clavier.
- Pas d'annulation/rétablissement (undo/redo).
- Le presse-papiers Couper/Copier/Coller opère à la granularité de la
  ligne entière (pas de sélection de texte arbitraire).
- La coloration syntaxique est fondée sur des règles lexicales simples par
  langage (mots-clés, chaînes, commentaires, etc.), pas sur une analyse
  syntaxique complète : certains cas rares peuvent être mal colorés (par
  exemple l'interpolation d'une variable à l'intérieur d'une chaîne entre
  guillemets en Bash).
- L'explorateur ne permet pas de créer, renommer ou supprimer des
  fichiers/dossiers ; il ne sert qu'à naviguer et ouvrir.
- Seul un module de compilation Java (`javac`) est fourni ; aucun outil de
  build (Maven, Gradle) n'est détecté ni invoqué, et aucun autre langage
  n'est pris en charge.

## 7. Références

Le comportement décrit ci-dessus est couvert par des scénarios de test au
format Gherkin, disponibles dans [`docs/features/`](features/) et exécutés
via `cargo test --test features` :

- `edition.feature` — édition, sauvegarde, couper/coller
- `menu.feature` — navigation et actions de la barre de menu
- `ouverture_dossier.feature` — dialogue « Ouvrir un dossier... » et
  reracinage de l'explorateur
- `affichage.feature` — visibilité des panneaux
- `coloration_syntaxique.feature` — détection du langage par extension
- `structure.feature` — extraction des symboles du panneau Structure
- `compilation.feature` — détection du module Java et compilation du projet

Pour le détail de l'architecture et des choix d'implémentation, voir
[`02-reditor-dat.md`](02-reditor-dat.md).
