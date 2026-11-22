#include "BackendClient.h"

#include <QGuiApplication>
#include <QDir>
#include <QColor>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QDebug>
#include <QPalette>
#include <QQuickStyle>
#include <QUrl>
#include <cstdio>
#include <algorithm>

namespace {
void applyDarkWhitePalette(QGuiApplication &app)
{
    QPalette palette = app.palette();
    const auto groups = {QPalette::Active, QPalette::Inactive, QPalette::Disabled};
    for (const QPalette::ColorGroup group : groups) {
        palette.setColor(group, QPalette::Window, QColor(QStringLiteral("#202124")));
        palette.setColor(group, QPalette::WindowText, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::Base, QColor(QStringLiteral("#25262b")));
        palette.setColor(group, QPalette::AlternateBase, QColor(QStringLiteral("#2c2d33")));
        palette.setColor(group, QPalette::Text, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::Button, QColor(QStringLiteral("#32343a")));
        palette.setColor(group, QPalette::ButtonText, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::BrightText, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::Highlight, QColor(QStringLiteral("#005a9e")));
        palette.setColor(group, QPalette::HighlightedText, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::PlaceholderText, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::Mid, QColor(QStringLiteral("#ffffff")));
        palette.setColor(group, QPalette::Midlight, QColor(QStringLiteral("#474950")));
        palette.setColor(group, QPalette::Dark, QColor(QStringLiteral("#18191d")));
        palette.setColor(group, QPalette::ToolTipBase, QColor(QStringLiteral("#25262b")));
        palette.setColor(group, QPalette::ToolTipText, QColor(QStringLiteral("#ffffff")));
    }
    app.setPalette(palette);
}
}

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
    applyDarkWhitePalette(app);

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
