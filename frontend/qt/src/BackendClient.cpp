#include "BackendClient.h"

#include <QDBusConnection>
#include <QDBusMessage>
#include <QDBusPendingCallWatcher>
#include <QDBusPendingReply>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QGuiApplication>
#include <QCursor>
#include <QUrl>

namespace {
const QString serviceName = QStringLiteral("io.github.radialkeys.Service");
const QString objectPath = QStringLiteral("/io/github/radialkeys/Service");
const QString interfaceName = QStringLiteral("io.github.radialkeys.Service1");
}

BackendClient::BackendClient(QObject *parent)
    : QObject(parent)
{
    m_statusTimer.setInterval(40);
    connect(&m_statusTimer, &QTimer::timeout, this, &BackendClient::pollStatus);
    m_statusTimer.start();

    m_heartbeatTimer.setInterval(1000);
    connect(&m_heartbeatTimer, &QTimer::timeout, this, &BackendClient::sendRendererHeartbeat);
    m_heartbeatTimer.start();
    sendRendererHeartbeat();
    refreshSlots();
}

BackendClient::~BackendClient()
{
    callMethod(QStringLiteral("RendererDisconnected"));
}

int BackendClient::cursorScreenIndex() const
{
    return m_cursorScreen ? QGuiApplication::screens().indexOf(m_cursorScreen) : 0;
}

void BackendClient::pollStatus()
{
    if (m_statusWatcher)
        return;

    auto message = QDBusMessage::createMethodCall(
        serviceName, objectPath, interfaceName, QStringLiteral("GetStatus"));
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, 1500), this);
    m_statusWatcher = watcher;
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, watcher]() {
                QDBusPendingReply<QString> reply = *watcher;
                m_statusWatcher = nullptr;
                if (reply.isError()) {
                    const bool changed = m_serviceAvailable || !m_errorMessage.isEmpty();
                    m_serviceAvailable = false;
                    m_rendererAvailable = false;
                    m_menuOpen = false;
                    m_shortcutRegistered = false;
                    m_errorMessage = reply.error().message();
                    watcher->deleteLater();
                    if (changed)
                        emit stateChanged();
                    return;
                }
                const bool wasOpen = m_menuOpen;
                const bool wasAvailable = m_serviceAvailable;
                m_serviceAvailable = true;
                updateFromStatus(reply.value());
                if (!wasOpen && m_menuOpen) {
                    m_cursorScreen = QGuiApplication::screenAt(m_cursorPosition);
                    if (!m_cursorScreen)
                        m_cursorScreen = QGuiApplication::primaryScreen();
                }
                if (!wasAvailable)
                    refreshSlots();
                m_errorMessage.clear();
                watcher->deleteLater();
                emit stateChanged();
            });
}

void BackendClient::refreshSlots()
{
    auto message = QDBusMessage::createMethodCall(
        serviceName, objectPath, interfaceName, QStringLiteral("GetSlots"));
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, 1500), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, watcher]() {
                QDBusPendingReply<QString> reply = *watcher;
                if (!reply.isError()) {
                    const auto document = QJsonDocument::fromJson(reply.value().toUtf8());
                    if (document.isArray()) {
                        m_slots.clear();
                        const auto array = document.array();
                        for (const auto &value : array) {
                            if (value.isObject())
                                m_slots.append(value.toObject().toVariantMap());
                        }
                        emit stateChanged();
                    }
                }
                watcher->deleteLater();
            });
}

void BackendClient::updateFromStatus(const QString &json)
{
    const auto document = QJsonDocument::fromJson(json.toUtf8());
    if (!document.isObject()) {
        m_serviceAvailable = false;
        m_errorMessage = tr("The backend returned invalid status data.");
        return;
    }

    const auto status = document.object();
    m_rendererAvailable = status.value(QStringLiteral("renderer_available")).toBool();
    m_menuOpen = status.value(QStringLiteral("menu_open")).toBool();
    m_activationEnabled = status.value(QStringLiteral("global_activation_enabled")).toBool();
    m_shortcutRegistered = status.value(QStringLiteral("global_shortcut_registered")).toBool();
    m_shortcut = status.value(QStringLiteral("shortcut")).toString(QStringLiteral("Ctrl+Shift+Y"));
    m_activationType = status.value(QStringLiteral("activation_type")).toString(QStringLiteral("keyboard"));
    m_mouseButton = status.value(QStringLiteral("mouse_button")).toString(QStringLiteral("back"));
    m_mouseHelperStatus = status.value(QStringLiteral("mouse_helper_status")).toString(QStringLiteral("Disabled"));
    if (!m_menuOpen && status.value(QStringLiteral("cursor_x")).isDouble()
        && status.value(QStringLiteral("cursor_y")).isDouble()) {
        m_cursorPosition = QPoint(status.value(QStringLiteral("cursor_x")).toInt(),
                                  status.value(QStringLiteral("cursor_y")).toInt());
    } else if (!m_menuOpen) {
        m_cursorPosition = QCursor::pos();
    }
}

void BackendClient::sendRendererHeartbeat()
{
    callMethod(QStringLiteral("RendererHeartbeat"));
}

void BackendClient::callMethod(const QString &method, const QVariantList &arguments)
{
    auto message = QDBusMessage::createMethodCall(serviceName, objectPath, interfaceName, method);
    message.setArguments(arguments);
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, 310000), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, watcher]() {
                QDBusPendingReply<> reply = *watcher;
                if (reply.isError()) {
                    m_errorMessage = reply.error().message();
                    emit stateChanged();
                }
                watcher->deleteLater();
            });
}

void BackendClient::enableGlobalActivation()
{
    m_errorMessage.clear();
    callMethod(QStringLiteral("SetGlobalActivationEnabled"), {true});
}

void BackendClient::disableGlobalActivation()
{
    callMethod(QStringLiteral("SetGlobalActivationEnabled"), {false});
}

void BackendClient::cancelMenu()
{
    callMethod(QStringLiteral("CancelMenu"));
}

void BackendClient::showMenu()
{
    callMethod(QStringLiteral("ShowMenu"));
}

void BackendClient::hideMenu()
{
    callMethod(QStringLiteral("HideMenu"));
}

void BackendClient::setSlot(int index, const QString &label, const QString &script, const QString &icon)
{
    if (index < 0 || index >= 8) {
        m_errorMessage = tr("Slot index is out of range.");
        emit stateChanged();
        return;
    }
    auto message = QDBusMessage::createMethodCall(
        serviceName, objectPath, interfaceName, QStringLiteral("SetSlotDetails"));
    message.setArguments({QVariant::fromValue(static_cast<uint>(index)), label, script, icon});
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, 10000), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, watcher]() {
                QDBusPendingReply<> reply = *watcher;
                if (reply.isError()) {
                    m_errorMessage = reply.error().message();
                    emit stateChanged();
                } else {
                    m_errorMessage.clear();
                    refreshSlots();
                }
                watcher->deleteLater();
            });
}

QString BackendClient::localPath(const QString &url) const
{
    const QUrl parsed(url);
    return parsed.isLocalFile() ? parsed.toLocalFile() : url;
}

QString BackendClient::fileUrl(const QString &path) const
{
    return path.isEmpty() ? QString() : QUrl::fromLocalFile(path).toString();
}

void BackendClient::setActivationDevice(const QString &device, const QString &button)
{
    auto message = QDBusMessage::createMethodCall(
        serviceName, objectPath, interfaceName, QStringLiteral("SetActivationDevice"));
    message.setArguments({device, button});
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, 10000), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, watcher]() {
                QDBusPendingReply<> reply = *watcher;
                if (reply.isError()) {
                    m_errorMessage = reply.error().message();
                    emit stateChanged();
                } else {
                    m_errorMessage.clear();
                }
                watcher->deleteLater();
            });
}

void BackendClient::activateSlot(int index)
{
    if (index < 0 || index >= 8)
        return;
    auto message = QDBusMessage::createMethodCall(
        serviceName, objectPath, interfaceName, QStringLiteral("ActivateSlot"));
    message.setArguments({QVariant::fromValue(static_cast<uint>(index))});
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, 10000), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, watcher]() {
                QDBusPendingReply<QString> reply = *watcher;
                if (reply.isError()) {
                    m_errorMessage = reply.error().message();
                    emit stateChanged();
                }
                watcher->deleteLater();
            });
}
