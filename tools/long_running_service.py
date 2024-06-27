import sys

STOP = False

print("Started")
print("Arguments: ", sys.argv[1:])
print("Commands: work, stop, ping")

while not STOP:
    command = input("")
    print("Received '{}'".format(command))

    # This is just to show some CPU usage when using attach
    if command == "work":
        index = 0
        for x in range(0, 10000000):
            index = x
            index = index * 2

        print("Worked")

    if command == "stop":
        STOP = True

    if command == "ping":
        print("pog")

print("Stopped")
