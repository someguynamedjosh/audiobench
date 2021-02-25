function transitionMenu(from, to) {
    document.getElementById(from).classList.add('hidden');
    setTimeout(() => {
        document.getElementById(to).classList.remove('hidden');
    }, 100);
}

function downloadURI(uri, name) {
    var link = document.createElement("a");
    // If you don't know the name or want to use
    // the webserver default set name = ''
    link.setAttribute('download', name);
    link.href = uri;
    document.body.appendChild(link);
    link.click();
    link.remove();
}

function linkClicked(name) {
    if (name === 'getting-started') {
        window.open('book/getting_started.html', '_blank');
    } else if (name === 'github') {
        window.open('https://github.com/joshua-maros/audiobench', '_blank');
    } else if (name == 'sponsor') {
        transitionMenu('links-main', 'links-sponsor');
    } else if (name === 'sponsor-github') {
        window.open('https://github.com/sponsors/joshua-maros', '_blank');
        transitionMenu('links-sponsor', 'links-main');
    } else if (name === 'sponsor-kofi') {
        window.open('https://ko-fi.com/joshua_maros', '_blank');
        transitionMenu('links-sponsor', 'links-main');
    }
}

function startDownload(from, artifactUrl) {
    let prettyArtifactName = '';
    let manualLink = document.getElementById('manual-link');
    manualLink.href = artifactUrl;
    manualLink.setAttribute('download', prettyArtifactName);
    downloadURI(artifactUrl, prettyArtifactName);
    document.getElementById(from).classList.add('hidden');
    setTimeout(() => {
        document.getElementById('download-menu-downloading').classList.remove('hidden');
    }, 100);
}

function makeDownloadButton(name, parentMenu, artifactName) {
    document.getElementById(name).addEventListener('click', event => {
        startDownload(parentMenu, 'https://github.com/joshua-maros/audiobench/releases/download/0.2.1/' + artifactName);
    });
}

window.addEventListener('DOMContentLoaded', event => {
    makeDownloadButton('btn-windows', 'download-menu-root', 'AudiobenchWindowsSetup.exe');
    makeDownloadButton('btn-macos', 'download-menu-root', 'AudiobenchMacOSSetup.pkg');
    makeDownloadButton('btn-linux', 'download-menu-root', 'AudiobenchLinuxSetup.sh');
});
