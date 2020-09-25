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
    let extension = artifactUrl.substr(artifactUrl.lastIndexOf('.') + 1);
    let prettyArtifactName = 'Audiobench.' + extension;
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
        startDownload(parentMenu, 'bin/' + artifactName);
    });
}

window.addEventListener('DOMContentLoaded', event => {
    document.getElementById('btn-windows').addEventListener('click', event => {
        transitionMenu('download-menu-root', 'download-menu-windows');
    });
    document.getElementById('btn-macos').addEventListener('click', event => {
        transitionMenu('download-menu-root', 'download-menu-macos');
    });
    document.getElementById('btn-linux').addEventListener('click', event => {
        transitionMenu('download-menu-root', 'download-menu-linux');
    });

    makeDownloadButton('btn-windows-standalone', 'download-menu-windows', 'Audiobench_Windows_x64_Standalone.exe');
    makeDownloadButton('btn-windows-vst3', 'download-menu-windows', 'Audiobench_Windows_x64_VST3.vst3');
    makeDownloadButton('btn-macos-standalone', 'download-menu-macos', 'Audiobench_MacOS_x64_Standalone.zip');
    makeDownloadButton('btn-macos-au', 'download-menu-macos', 'Audiobench_MacOS_x64_AU.zip');
    makeDownloadButton('btn-macos-vst3', 'download-menu-macos', 'Audiobench_MacOS_x64_VST3.zip');
    makeDownloadButton('btn-linux-standalone', 'download-menu-linux', 'Audiobench_Linux_x64_Standalone.bin');
    makeDownloadButton('btn-linux-vst3', 'download-menu-linux', 'Audiobench_Linux_x64_VST3.so');
});
