#include "BackendClient.h"

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QDebug>
#include <QUrl>
#include <cstdio>

int main(int argc, char *argv[])
{
    qInfo() << "Starting Radial Keys renderer";
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Radial Keys"));
    QGuiApplication::setOrganizationDomain(QStringLiteral("io.github.radialkeys"));
    QGuiApplication::setQuitOnLastWindowClosed(false);

    BackendClient backend;
    QQmlApplicationEngine engine;
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
