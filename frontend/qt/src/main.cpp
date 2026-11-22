#include "BackendClient.h"

#include <QGuiApplication>
#include <QDir>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QDebug>
#include <QQuickStyle>
#include <QUrl>
#include <cstdio>
#include <algorithm>

int main(int argc, char *argv[])
{
    qInfo() << "Starting Radial Keys renderer";
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Radial Keys"));
    QGuiApplication::setOrganizationDomain(QStringLiteral("io.github.radialkeys"));
    QGuiApplication::setQuitOnLastWindowClosed(false);

    QQmlApplicationEngine engine;
    const QString desktop = qEnvironmentVariable("XDG_CURRENT_DESKTOP");
    const bool isKdeSession = desktop.contains(QStringLiteral("KDE"), Qt::CaseInsensitive)
        || !qEnvironmentVariableIsEmpty("KDE_FULL_SESSION");
    const bool hasExplicitStyle = !qEnvironmentVariableIsEmpty("QT_QUICK_CONTROLS_STYLE");
    const QStringList importPaths = engine.importPathList();
    const bool hasKdeStyle = std::any_of(
        importPaths.cbegin(), importPaths.cend(), [](const QString &path) {
            return QDir(path + QStringLiteral("/org/kde/desktop")).exists();
        });
    if (isKdeSession && !hasExplicitStyle && hasKdeStyle) {
        QQuickStyle::setStyle(QStringLiteral("org.kde.desktop"));
        qInfo() << "Using KDE desktop Qt Quick Controls style";
    }

    BackendClient backend;
    QObject::connect(&engine, &QQmlApplicationEngine::warnings, &engine,
                     [](const QList<QQmlError> &warnings) {
                         for (const auto &warning : warnings) {
                             const auto message = warning.toString().toUtf8();
                             std::fprintf(stderr, "%s\n", message.constData());
                             qWarning().noquote() << warning.toString();
                         }
                     });
    engine.rootContext()->setContextProperty(QStringLiteral("backend"), &backend);
    engine.loadFromModule(QStringLiteral("RadialKeys.Frontend"), QStringLiteral("Main"));
    if (engine.rootObjects().isEmpty()) {
        std::fprintf(stderr, "Radial Keys: QML root object failed to load.\n");
        qCritical() << "The QML frontend failed to create its root window";
        return 1;
    }
    qInfo() << "Radial Keys renderer window created";

    return app.exec();
}
