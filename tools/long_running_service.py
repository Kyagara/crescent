STOP = False

print("Started long running service")

while not STOP:
    command = input("")
    
    # This is just to show some CPU usage when using attach
    if command == "work":
        print("Working")
        
        index = 0
        for x in range(0, 1000000):
            index = x

        print("Worked")
        
    if command == "stop":
        STOP = True

    if command == "ping":
        print("pog")

print("Stopping")
