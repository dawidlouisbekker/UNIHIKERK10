import serial
import serial.tools.list_ports
import time

BAUDRATE = 115200
PROBE_TIME = 1.5  # seconds to wait for data


def probe_port(port_name):
    try:
        ser = serial.Serial(port_name, BAUDRATE, timeout=0.5)

        start = time.time()
        received = False

        while time.time() - start < PROBE_TIME:
            data = ser.readline()
            if data:
                received = True
                break

        ser.close()

        if received:
            return "ACTIVE"
        else:
            return "OPEN (no data)"

    except Exception:
        return "UNAVAILABLE"


def list_interfaces_with_status():
    ports = list(serial.tools.list_ports.comports())

    if not ports:
        print("No interfaces found.")
        return []

    print("\nInterfaces:\n")

    results = []

    for i, port in enumerate(ports):
        status = probe_port(port.device)

        print(f"[{i}] {port.device}")
        print(f"     Description : {port.description}")
        print(f"     Status      : {status}")
        print()

        results.append((port.device, status))

    return ports


def select_interface(ports):
    while True:
        try:
            choice = input("Select interface index: ").strip()
            index = int(choice)

            if 0 <= index < len(ports):
                return ports[index].device
            else:
                print("Invalid selection.")
        except:
            print("Enter a valid number.")


def main():
    ports = list_interfaces_with_status()

    if not ports:
        return

    port = select_interface(ports)

    print(f"\nOpening {port}...\n")

    try:
        ser = serial.Serial(port, BAUDRATE, timeout=1)

        print("Reading (Ctrl+C to exit):\n")

        while True:
            data = ser.readline()
            if data:
                print(data.decode(errors="ignore").strip())

    except KeyboardInterrupt:
        print("\nExiting...")
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    main()