# Ferma 1.1.0 — note de versiune

Schimbările realizate în această conversație față de versiunea 1.0.1.

## Fișa bovinei

- Fișa are tab-uri separate pentru **Date generale**, **Monte** și **Fătări**. Fereastra își păstrează dimensiunea la schimbarea tab-urilor.
- Rasa, sexul, categoria și starea sunt în **Date generale**. Contoarele de monte și fătări nu mai ocupă loc în această secțiune.
- **Mama** înlocuiește „Fătarea de origine” și afișează numai crotalia. Trecerea cursorului peste crotalie deschide un panou cu datele mamei, fără să deplaseze conținutul fișei.
- Crotaliile vițeilor din tab-ul **Fătări** deschid același panou de consultare la apăsare; panoul rămâne afișat cât timp cursorul este pe crotalie sau pe panou. Panourile pot fi accesate și din tastatură.

## Monte, fătări și viței

- Montele și fătările pot fi adăugate și editate din tab-urile bovinei. Când un formular este deschis din fișa unei bovine, animalul respectiv rămâne fixat, iar câmpul „Mama” este ascuns dacă fișa aparține mamei.
- O fătare poate fi înregistrată fără vițel. Un vițel deja înregistrat poate fi asociat la salvare sau ulterior. În editarea fătării, asocierea poate fi schimbată ori eliminată fără ștergerea vițelului din registru.
- O fătare poate avea mai mulți viței. Butonul **Adaugă un vițel** rămâne disponibil și când există deja viței, iar în formularul de editare poate fi adăugată o nouă asociere sau schimbată separat una existentă.
- Formularul **Adaugă o bovină** are câmpul opțional **Mama**. Dacă este completat, vițelul și fătarea se salvează împreună la data nașterii. O fătare deja înregistrată pentru aceeași mamă și zi este reutilizată.
- Listele de selecție a bovinelor filtrează după orice fragment din crotalie. Mama este exclusă din lista de viței eligibili, iar erorile de asociere indică precis cauza.
- Salvarea legăturilor dintre vițel și fătare este atomică și poate fi anulată sau refăcută prin **Undo/Redo**.

## Istoricul evenimentelor

- Paginile **Monte** și **Fătări** au filtre combinabile pentru an, lună, fragmente de crotalie, rasă și prezența animalului asociat. Fătările pot fi filtrate și după sexul vițelului.
- Tabelul fătărilor afișează câte un rând pentru fiecare vițel: specia, rasa, sexul, data nașterii, crotalia vițelului și crotalia mamei. Fătările fără vițel asociat rămân vizibile, iar sumarul distinge între fătări și vițeii afișați.

## Pregătirea versiunilor

- `package.json` este sursa pentru numărul versiunii. `npm.cmd run version:get` îl afișează; `npm.cmd run version:bump -- big|medium|small` îl mărește și afișează versiunea veche și cea nouă.
- `npm.cmd run version:set -- <versiune>` setează o valoare exactă. Comenzile sincronizează versiunea din configurația Tauri și fișierul Cargo, inclusiv fișierele lock.
- Actualizările aplicației rămân manuale: noul instalator se generează cu `npm.cmd run build:desktop` și se instalează folosind același cont Windows pentru a păstra datele fermei.
