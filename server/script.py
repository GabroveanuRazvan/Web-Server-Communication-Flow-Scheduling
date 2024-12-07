def compare_files(file1_path, file2_path):
    try:
        with open(file1_path, 'rb') as file1, open(file2_path, 'rb') as file2:
            position = 0
            while True:
                char1 = file1.read(1)
                char2 = file2.read(1)

                # Dacă ambele fișiere au ajuns la sfârșit
                if not char1 and not char2:
                    print("Fișierele sunt identice.")
                    break

                # Dacă doar unul dintre fișiere a ajuns la sfârșit
                if not char1 or not char2:
                    print(f"Fișierele au lungimi diferite. Diferență la poziția {position}.")
                    break

                # Compară caracterele
                if char1 != char2:
                    print(f"Fișierele diferă la poziția {position}: '{char1}' vs '{char2}'")
                    break

                position += 1

    except FileNotFoundError as e:
        print(f"Eroare: {e}")
    except Exception as e:
        print(f"Eroare neașteptată: {e}")

# Exemple de folosire
file1_path = "/tmp/tmpfs/__images_4k__4k4.jpg"
file2_path = "./web_files/images_4k/4k4.jpg"

compare_files(file1_path, file2_path)
