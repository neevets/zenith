import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
	console.log('Congratulations, your extension "zenith-vscode" is now active!');

	let disposable = vscode.commands.registerCommand('zenith-vscode.helloWorld', () => {
		vscode.window.showInformationMessage('Hello World from Zenith!');
	});

	context.subscriptions.push(disposable);
}

export function deactivate() {}
