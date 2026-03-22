# GUI

For the graphical user interface (GUI), React is used as the front-end framework. The GUI code is located in the `gui`
directory.

### Provider

The provider admin interface is setup for development using Vite. To start the development server, navigate to the `gui`
directory and run:

```bash
cd gui
npm install
export ENV_FILE=./../../static/envs/.env.provider.core && npm run dev -w provider
```

### Consumer

The consumer admin interface is setup for development using Vite. To start the development server, navigate to the `gui`
directory and run:

```bash
cd gui
npm install
export ENV_FILE=./../../static/envs/.env.consumer.core && npm run dev -w consumer
```


### Building for Production

To build the GUI for production, you can run the following commands from the `gui` directory

```bash
cd gui
npm install
export ENV_FILE=./../../static/envs/.env.provider.core && npm run build -w provider
export ENV_FILE=./../../static/envs/.env.consumer.core && npm run build -w consumer
export ENV_FILE=./../../static/envs/.env.provider.core && npm run build -w business
```

The built files will be located in the `dist` directory within each respective project folder (`provider`, `consumer`,
`business`).
But they are meant to be served by a web server in axum, not directly.
