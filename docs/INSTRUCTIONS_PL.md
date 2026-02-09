# SilliReminder — Instrukcja (Polski)

SilliReminder to prosta aplikacja do przypomnień na Windows. Zapisuje przypomnienia lokalnie na Twoim komputerze (baza SQLite) i wyświetla powiadomienia systemowe, gdy przypomnienie stanie się aktualne.

## Szybki start (3 kroki)

1. **Otwórz aplikację**
   - Jeśli widzisz okno z nagłówkami **Ustawienia**, **Dodaj**, **Zaplanowane**: wszystko działa.
   - Jeśli nie widzisz okna: sprawdź **ikonę SilliReminder w zasobniku systemowym** (obok zegara). Kliknij prawym → **Otwórz**.
   - Jeśli nie widzisz ikony: kliknij strzałkę **`^` (Pokaż ukryte ikony)** obok zegara i poszukaj tam.

2. **Dodaj przypomnienie (sekcja „Dodaj”)**
   - Kliknij pole z datą po lewej (np. `2026-02-12` z ikoną kalendarza) i wybierz datę.
   - Kliknij pole **„Notatka…”** i wpisz treść.
   - Kliknij przycisk **„Dodaj”** po prawej.

3. **Poczekaj na powiadomienie**
   - Gdy nadejdzie czas, Windows pokaże powiadomienie.
   - Jeśli nic się nie pojawia, zobacz sekcję „Rozwiązywanie problemów”.

## Jak działa aplikacja (ważne)

### Aplikacja może działać „w trayu”
- Zamknięcie okna może tylko **schować aplikację do zasobnika** zamiast ją zamykać.
- Aby ponownie otworzyć okno: prawy przycisk na ikonie w trayu → **Otwórz**.
- Aby wyjść całkowicie: prawy przycisk na ikonie w trayu → **Zamknij**.

### Powiadomienia
- Gdy przypomnienie stanie się aktualne, aplikacja uruchamia powiadomienie systemowe.
- Aplikacja stara się nie spamować tym samym przypomnieniem po restarcie (pamięta, co już ogłosiła).

### Start z Windowsem (sekcja „Ustawienia”)
- Zaznacz checkbox **„Włącz podczas włączania systemu”**, jeśli aplikacja ma startować razem z Windowsem.
- Działa dla bieżącego użytkownika (bez uprawnień administratora).

## Zarządzanie przypomnieniami

### Lista zaplanowanych (sekcja „Zaplanowane”)
- Przypomnienia są widoczne pod nagłówkiem **„Zaplanowane”**.
- Format na liście wygląda jak: `YYYY-MM-DD - Twoja notatka`.

### Usuwanie
- Każde przypomnienie na liście ma po prawej **czerwony przycisk „X”**.
- Kliknij **X**, aby usunąć przypomnienie.

## Gdzie są zapisane dane

Aplikacja zapisuje dane per użytkownik (bez administratora):

- Baza (Twoje przypomnienia): `%LOCALAPPDATA%\SilliReminder\data\silli_reminder.db`
- Ustawienia: `%LOCALAPPDATA%\SilliReminder\settings.sillisettings`

## Odinstalowanie (czyste usunięcie)

Podczas odinstalowania możesz zaznaczyć:

- **Usuń bazę danych (przypomnienia)** — usuwa wszystkie przypomnienia.
- **Usuń ustawienia (preferencje)** — resetuje ustawienia aplikacji.

Odinstalowanie usuwa też wpis autostartu (jeśli istnieje):
- `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\SilliReminder`

## Rozwiązywanie problemów (najczęstsze)

### „Zainstalowałem, ale nie widzę okna”
1. Sprawdź zasobnik (obok zegara).
2. Kliknij prawym na ikonę SilliReminder → **Otwórz**.
3. Jeśli nie ma ikony: kliknij strzałkę **`^` (Pokaż ukryte ikony)** obok zegara.
4. Jeśli nadal nie ma ikony: uruchom aplikację ponownie z menu Start.

### „Nie ma powiadomień”
1. Otwórz **Ustawienia Windows → System → Powiadomienia**.
2. Upewnij się, że powiadomienia są włączone globalnie.
3. Znajdź **SilliReminder** na liście aplikacji i włącz powiadomienia.
4. Tryb skupienia / Nie przeszkadzać może blokować powiadomienia — wyłącz tymczasowo.

### „Odinstalowanie nie usuwa bazy/ustawień”
- Aplikacja może nadal działać w trayu i blokować pliki.
- Prawy na ikonie w trayu → **Zamknij**, potem uruchom odinstalowanie ponownie.

### „Windows SmartScreen ostrzega”
- To może się zdarzyć dla niepodpisanych aplikacji lub nowych wersji.
- Pobieraj tylko z zaufanego, oficjalnego linku.

## Wskazówki (żeby przypomnienia były pewne)
- Utrzymuj poprawną datę i godzinę w Windows.
- Gdy komputer śpi/hibernuje, powiadomienia mogą się opóźniać.

## Przykłady dla księgowych

W polu **„Notatka…”** wpisuj krótko, ale jednoznacznie. Dobre przykłady:

- `VAT — wysyłka deklaracji`
- `Wypłaty — przelewy`
- `Koniec miesiąca — zamknięcie ksiąg`
- `Termin faktury — klient XYZ`

Wskazówka: zacznij od czynności (Wyślij/Zapłać/Wykonaj/Zamknij), potem dopiero temat.

## Kopia zapasowa (bardzo zalecane)

Przypomnienia są zapisane tylko na tym komputerze (lokalna baza). Po reinstalacji Windows lub zmianie komputera nie przeniosą się automatycznie.

Aby zrobić kopię przypomnień, skopiuj ten plik w bezpieczne miejsce:

- `%LOCALAPPDATA%\SilliReminder\data\silli_reminder.db`

---
