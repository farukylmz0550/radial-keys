#pragma once

#include <QObject>
#include <QPoint>
#include <QPointer>
#include <QScreen>
#include <QTimer>
#include <QVariantList>

class QDBusPendingCallWatcher;

class BackendClient final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool serviceAvailable READ serviceAvailable NOTIFY stateChanged)
    Q_PROPERTY(bool rendererAvailable READ rendererAvailable NOTIFY stateChanged)
    Q_PROPERTY(bool menuOpen READ menuOpen NOTIFY stateChanged)
    Q_PROPERTY(bool activationEnabled READ activationEnabled NOTIFY stateChanged)
    Q_PROPERTY(bool shortcutRegistered READ shortcutRegistered NOTIFY stateChanged)
    Q_PROPERTY(QString shortcut READ shortcut NOTIFY stateChanged)
    Q_PROPERTY(QString activationType READ activationType NOTIFY stateChanged)
    Q_PROPERTY(QString mouseButton READ mouseButton NOTIFY stateChanged)
    Q_PROPERTY(QString mouseHelperStatus READ mouseHelperStatus NOTIFY stateChanged)
    Q_PROPERTY(QString errorMessage READ errorMessage NOTIFY stateChanged)
    Q_PROPERTY(QPoint cursorPosition READ cursorPosition NOTIFY stateChanged)
    Q_PROPERTY(int cursorScreenIndex READ cursorScreenIndex NOTIFY stateChanged)
    Q_PROPERTY(QVariantList menuSlots READ menuSlots NOTIFY stateChanged)
    Q_PROPERTY(QVariantList builtinMacros READ builtinMacros NOTIFY stateChanged)
    Q_PROPERTY(QVariantList runningMacros READ runningMacros NOTIFY stateChanged)
    Q_PROPERTY(QVariantList macroResults READ macroResults NOTIFY stateChanged)
    Q_PROPERTY(bool macroSingleInstance READ macroSingleInstance NOTIFY stateChanged)
    Q_PROPERTY(int macroTimeoutSeconds READ macroTimeoutSeconds NOTIFY stateChanged)
    Q_PROPERTY(bool microphoneMuted READ microphoneMuted NOTIFY stateChanged)
    Q_PROPERTY(bool microphoneStateKnown READ microphoneStateKnown NOTIFY stateChanged)

public:
    explicit BackendClient(QObject *parent = nullptr);
    ~BackendClient() override;

    bool serviceAvailable() const { return m_serviceAvailable; }
    bool rendererAvailable() const { return m_rendererAvailable; }
    bool menuOpen() const { return m_menuOpen; }
    bool activationEnabled() const { return m_activationEnabled; }
    bool shortcutRegistered() const { return m_shortcutRegistered; }
    QString shortcut() const { return m_shortcut; }
    QString activationType() const { return m_activationType; }
    QString mouseButton() const { return m_mouseButton; }
    QString mouseHelperStatus() const { return m_mouseHelperStatus; }
    QString errorMessage() const { return m_errorMessage; }
    QPoint cursorPosition() const { return m_cursorPosition; }
    int cursorScreenIndex() const;
    QVariantList menuSlots() const { return m_slots; }
    QVariantList builtinMacros() const { return m_builtinMacros; }
    QVariantList runningMacros() const { return m_runningMacros; }
    QVariantList macroResults() const { return m_macroResults; }
    bool macroSingleInstance() const { return m_macroSingleInstance; }
    int macroTimeoutSeconds() const { return m_macroTimeoutSeconds; }
    bool microphoneMuted() const { return m_microphoneMuted; }
    bool microphoneStateKnown() const { return m_microphoneStateKnown; }

    Q_INVOKABLE void enableGlobalActivation();
    Q_INVOKABLE void disableGlobalActivation();
    Q_INVOKABLE void cancelMenu();
    Q_INVOKABLE void showMenu();
    Q_INVOKABLE void hideMenu();
    Q_INVOKABLE void setSlot(int index, const QString &label, const QString &script, const QString &icon);
    Q_INVOKABLE void setSlotPreset(int index, const QString &label, const QString &presetId, const QVariantList &parameters, const QString &icon);
    Q_INVOKABLE void setSlotPython(int index, const QString &label, const QString &source, const QString &icon);
    Q_INVOKABLE void activateSlot(int index);
    Q_INVOKABLE void acceptScriptUpdate(int index);
    Q_INVOKABLE void setMacroOptions(bool singleInstance, int timeoutSeconds);
    Q_INVOKABLE void terminateMacro(const QString &id);
    Q_INVOKABLE void terminateSlotMacros(int index);
    Q_INVOKABLE int presetIndex(const QString &presetId) const;
    Q_INVOKABLE QString localPath(const QString &url) const;
    Q_INVOKABLE QString fileUrl(const QString &path) const;
    Q_INVOKABLE QString readPythonSource(const QString &path) const;
    Q_INVOKABLE void setActivationDevice(const QString &device, const QString &button);

signals:
    void stateChanged();

private slots:
    void pollStatus();
    void pollMacroStatus();
    void sendRendererHeartbeat();

private:
    void callMethod(const QString &method, const QVariantList &arguments = {});
    void updateFromStatus(const QString &json);
    void refreshSlots();
    void refreshBuiltinMacros();

    QTimer m_statusTimer;
    QTimer m_heartbeatTimer;
    QTimer m_macroStatusTimer;
    QPointer<QDBusPendingCallWatcher> m_statusWatcher;
    bool m_serviceAvailable = false;
    bool m_rendererAvailable = false;
    bool m_menuOpen = false;
    bool m_activationEnabled = false;
    bool m_shortcutRegistered = false;
    QString m_shortcut = QStringLiteral("Ctrl+Shift+Y");
    QString m_activationType = QStringLiteral("keyboard");
    QString m_mouseButton = QStringLiteral("back");
    QString m_mouseHelperStatus = QStringLiteral("Disabled");
    QString m_errorMessage;
    QPoint m_cursorPosition;
    QScreen *m_cursorScreen = nullptr;
    QVariantList m_slots;
    QVariantList m_builtinMacros;
    QVariantList m_runningMacros;
    QVariantList m_macroResults;
    bool m_macroSingleInstance = true;
    int m_macroTimeoutSeconds = -1;
    bool m_microphoneMuted = false;
    bool m_microphoneStateKnown = false;
};
