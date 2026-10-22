# Premiers pas en français

REMAIN est une application de bureau qui permet de gérer et de réunir les
rapports de projets et de sous-projets. Elle analyse les indicateurs fournis
dans des fichiers Excel et exporte des rapports PDF assemblés, des résultats
Excel et des synthèses HTML imprimables. Les groupes et les catégories sont
facultatifs et indépendants.

## Installation

Le [guide d’installation](../installation.md) décrit l’installation, la mise à
jour et la désinstallation sous Windows, Linux et macOS.

## Premier rapport

1. Lancez REMAIN et sélectionnez **Français**.
2. Choisissez **Créer un espace de démonstration → Rapports simples**, puis un dossier.
3. Cliquez sur **Analyser les données**.
4. Examinez les indicateurs par groupe, catégorie ou projet et essayez les graphiques.
5. Cliquez sur **Exporter les rapports**.
6. Ouvrez le dossier de sortie, puis `index.html` ou `report.pdf`.

La démonstration contient uniquement des données fictives. Pour vos propres
données, choisissez **Nouvel espace de travail**. **Ouvrir un espace de travail**
sélectionne le dossier contenant `workspace.json`. Dans **Modifier la configuration**,
définissez les documents attendus et leurs motifs de noms de fichiers. Au moins
un PDF est requis ; le classeur KPI est facultatif. La **Liste des projets** se
modifie séparément et peut être importée depuis CSV/JSON. Les soumissions requises
manquantes sont signalées. Un export partiel est possible dès qu’au moins un
projet a fourni tous ses documents requis ; il porte la mention « Rapport incomplet ».
Les rapports globaux, par groupe et par catégorie commencent
par un sommaire avec les numéros de page.

Les références détaillées sont disponibles en anglais :

- [Tutoriels](../tutorials.md)
- [Préparer les fichiers](../workspaces.md)
- [Configurer les indicateurs et les langues](../configuration.md)
- [Comprendre les exports](../outputs.md)
- [Résoudre les problèmes](../troubleshooting.md)
