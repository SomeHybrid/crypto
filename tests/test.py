import asyncio, sys, ssl

LISTEN_PORT = 7500
DST_PORT = 443
DST_HOST = "https://www.wikipedia.com"
#DST_HOST = "neverssl.com"

async def pipe(reader, writer):
    try:
        while not reader.at_eof():
            data = await reader.read(2048)
            print(f"Received {data!r}")
            writer.write(data)
    finally:
        writer.close()

async def handle_client(client_reader, client_writer):
    try:
        # connect to database server
        ssl_context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
        ssl_context.check_hostname = True
        server_reader, server_writer = await asyncio.open_connection(
            DST_HOST, DST_PORT, ssl=ssl_context)
        print("Connected to server")
        pipe1 = pipe(client_reader, server_writer)
        pipe2 = pipe(server_reader, client_writer)
        await asyncio.gather(pipe1, pipe2)
    except Exception as e:
        # printing the error message
        print(f"Error connecting to server: {sys.exc_info()[0]}")
        print(f"{e}")
    finally:
        print("Closing connection")
        client_writer.close()

async def main():
    server = await asyncio.start_server(handle_client, '127.0.0.1', LISTEN_PORT)

    print('Serving on {}'.format(server.sockets[0].getsockname()))
    try:
        await server.serve_forever()
    except KeyboardInterrupt:
        pass

    server.close()
    await server.wait_closed()

asyncio.run(main())
